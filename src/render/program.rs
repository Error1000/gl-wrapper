use crate::render::shader::*;
use crate::unwrap_result_or_ret;
use gl::types::*;
use std::{cell::RefCell, collections::HashMap, ops::{Deref, DerefMut}};
use std::convert::{TryFrom, TryInto};
use std::ffi::CString;
use std::ptr;
use std::str;

// Only one program can be bound at a time
const BOUNCER:  RefCell<bool> = RefCell::new(false);


pub struct BoundProgram<'a>(&'a mut Program);

impl Deref for BoundProgram<'_>{
    type Target = Program;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for BoundProgram<'_>{
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Drop for BoundProgram<'_>{
    fn drop(&mut self){ *(BOUNCER.borrow_mut()) = false; }
}

pub struct UnboundProgram(Program);

impl UnboundProgram{

    pub fn bind(&mut self) -> Option<BoundProgram>{
        if *(BOUNCER.borrow()) == false{
            *(BOUNCER.borrow_mut()) = true;
            self.0.bind_program();
            Some(BoundProgram(&mut self.0))
        } else {
            None
        }
    }
}

pub struct Program {
    id: GLuint,
    /// TODO: Maybe optimise this as realistically there aren't going to be more than 100 entries
    /// in either of these hash maps so the 1 in O(1) becomes pretty big in comparison to a simple array (because of the small size)
    uniform_ids: HashMap<String, GLuint>,
    attrib_ids: HashMap<String, GLuint>,
}

impl Drop for Program {
    fn drop(self: &mut Self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

impl Program {
    pub fn new(shaders: &[&ShaderBase]) -> Result<UnboundProgram, String> {
        let r = Program {
            id: unsafe { gl::CreateProgram() },
            uniform_ids: HashMap::new(),
            attrib_ids: HashMap::new(),
        };
        r.bind_program();
        // Attach all shaders to program
        shaders
            .iter()
            .for_each(|s| unsafe { gl::AttachShader(r.id, s.get_id()) });

        // Link
        unsafe {
            gl::LinkProgram(r.id);
        }

        let mut program_compiled: GLint = gl::FALSE.into();
        unsafe {
            gl::GetProgramiv(r.id, gl::LINK_STATUS, &mut program_compiled);
        }

        if program_compiled != gl::TRUE.into() {
            let mut len: i32 = 0;
            unsafe {
                gl::GetProgramiv(r.id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let new_len: usize = match len.try_into() {
                Ok(val) => val,
                Err(_) => return Err(String::from("")),
            };
            let mut buf = Vec::<u8>::with_capacity(new_len);
            unsafe {
                gl::GetProgramInfoLog(r.id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            }
            match str::from_utf8(&buf) {
                Ok(msg) => return Err(String::from(msg)),
                Err(std::str::Utf8Error { .. }) => {
                    return Err(String::from("ProgramInfoLog not a valid utf8 string"))
                }
            };
        }

        // Detach all shaders from program
        shaders
            .iter()
            .for_each(|s| unsafe { gl::DetachShader(r.id, s.get_id()) });
        Ok(UnboundProgram(r))
    }

    fn bind_program(self: &Self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    // General loader of values (private)
    /// TODO: Should probably get rid of inline(always) but i know this function is only used in like 2 spots and there is no real reason not to inline it
    #[inline(always)]
    fn get_id_of(
        self: &Self,
        name: &str,
        get_location: unsafe fn(GLuint, *const GLchar) -> GLint,
    ) -> Result<u32, &str> {
        let id = {
            let cname = unwrap_result_or_ret!(CString::new(name.as_bytes()), Err("Invalid name!"));
            unsafe { get_location(self.id, cname.as_ptr() as *const GLchar) }
        };
        if id < 0 {
            return Err("Could not find id!");
        }
        let id: u32 = unwrap_result_or_ret!(id.try_into(), Err("Id returned from opengl is too big to fit in return type, faulty opengl implementation!"));
        Ok(id)
    }

    pub fn load_uniform(self: &mut Self, name: String) -> Result<(), String> {
        // Check if already loaded, glGetUniformLocation can be pretty damn slow and a simple contains_key, especially on a hashbrown is probablly way faster
        if !self.uniform_ids.contains_key(&name) {
            let u_id = self.get_id_of(&name, gl::GetUniformLocation)?;
            self.uniform_ids.insert(name, u_id);
        }
        Ok(())
    }

    pub fn load_sampler(self: &mut Self, name: String) -> Result<(), String> {
        self.load_uniform(name)
    }

    pub fn load_attribute(self: &mut Self, name: String) -> Result<(), String> {
        // Check if already loaded, glGetUniformLocation can be pretty damn slow and a simple contains_key, especially on a hashbrown is probably way faster
        if !self.attrib_ids.contains_key(&name) {
            let a_id = self.get_id_of(&name, gl::GetAttribLocation)?;
            self.attrib_ids.insert(name, a_id);
        }
        Ok(())
    }

    /// Warning: Does NOT support arrays of uniforms/attributes/samplers!
    pub fn auto_load_all(self: &mut Self, buf_size: GLushort) -> Result<(), String> {
        let mut count: GLint = 0;
        let mut name = vec![0_u8; buf_size.into()];

        unsafe { gl::GetProgramiv(self.id, gl::ACTIVE_ATTRIBUTES, &mut count) };
        let count = unwrap_result_or_ret!(
            GLuint::try_from(count),
            Err(format!("Invalid number of attributes: {}", count))
        );
        for i in 0..count {
            let nam: &[u8] = {
                let mut typ: GLenum = 0;
                let mut siz: GLint = 0;
                let mut actual_len: GLsizei = 0;
                unsafe {
                    gl::GetActiveAttrib(
                        self.id,
                        unwrap_result_or_ret!(
                            i.try_into(),
                            Err(format!("Invalid attribute id: {}", i))
                        ),
                        buf_size.into(),
                        &mut actual_len,
                        &mut siz,
                        &mut typ,
                        name.as_mut_ptr() as *mut GLchar,
                    )
                };
                &name[0..unwrap_result_or_ret!(actual_len.try_into(), Err("Opengl returned negative name size for attribute, faulty opengl implementation!".to_owned()))]
            };
            self.attrib_ids.insert(
                String::from(unwrap_result_or_ret!(
                    str::from_utf8(nam),
                    Err(format!("Invalid attribute name: {:?}", nam))
                )),
                i,
            );
        }

        let mut count: GLint = 0;

        unsafe { gl::GetProgramiv(self.id, gl::ACTIVE_UNIFORMS, &mut count) };
        let count = unwrap_result_or_ret!(
            GLuint::try_from(count),
            Err(format!("Invalid number of uniforms: {}", count))
        );
        for i in 0..count {
            let nam: &[u8] = {
                let mut typ: GLenum = 0;
                let mut siz: GLint = 0;
                let mut actual_len: GLsizei = 0;
                unsafe {
                    gl::GetActiveUniform(
                        self.id,
                        unwrap_result_or_ret!(
                            i.try_into(),
                            Err(format!("Invalid uniform id: {}", i))
                        ),
                        buf_size.into(),
                        &mut actual_len,
                        &mut siz,
                        &mut typ,
                        name.as_mut_ptr() as *mut GLchar,
                    )
                };
                &name[0..unwrap_result_or_ret!(actual_len.try_into(), Err("Opengl returned negative name size for uniform, faulty opengl implementation!".to_owned()))]
            };
            self.uniform_ids.insert(
                String::from(unwrap_result_or_ret!(
                    str::from_utf8(nam),
                    Err(format!("Invalid attribute name: {:?}", nam))
                )),
                i,
            );
        }

        Ok(())
    }

    #[inline]
    pub fn clear_all_loaded(self: &mut Self) {
        self.uniform_ids.clear();
        self.attrib_ids.clear();
    }

    #[inline]
    pub fn set_uniform_i32(self: &mut Self, id: GLint, val: i32) {
        unsafe {
            gl::Uniform1i(id, val);
        }
    }

    #[inline]
    pub fn set_uniform_u32(self: &mut Self, id: GLint, val: u32) {
        unsafe {
            gl::Uniform1ui(id, val);
        }
    }

    #[inline]
    pub fn set_uniform_f32(self: &mut Self, id: GLint, val: f32) {
        unsafe {
            gl::Uniform1f(id, val);
        }
    }

    #[inline]
    pub fn set_uniform_vec3_f32(self: &mut Self, id: GLint, val: [f32; 3]) {
        unsafe {
            gl::Uniform3fv(id, 1, val.as_ptr());
        }
    }

    #[inline]
    pub fn set_uniform_vec3_i32(self: &mut Self, id: GLint, val: [i32; 3]) {
        unsafe {
            gl::Uniform3iv(id, 1, val.as_ptr());
        }
    }

    #[inline]
    pub fn set_uniform_vec3_u32(self: &mut Self, id: GLint, val: [u32; 3]) {
        unsafe {
            gl::Uniform3uiv(id, 1, val.as_ptr());
        }
    }

    #[inline]
    pub fn set_uniform_vec2_f32(self: &mut Self, id: GLint, val: [f32; 2]) {
        unsafe {
            gl::Uniform2fv(id, 1, val.as_ptr());
        }
    }

    #[inline]
    pub fn set_uniform_vec2_i32(self: &mut Self, id: GLint, val: [i32; 2]) {
        unsafe {
            gl::Uniform2iv(id, 1, val.as_ptr());
        }
    }

    #[inline]
    pub fn set_uniform_vec2_u32(self: &mut Self, id: GLint, val: [u32; 2]) {
        unsafe {
            gl::Uniform2uiv(id, 1, val.as_ptr());
        }
    }

    #[inline]
    pub fn set_uniform_mat3_f32(self: &mut Self, id: GLint, val: &[f32; 3 * 3]) {
        unsafe {
            gl::UniformMatrix3fv(id, 1, gl::FALSE, val.as_ptr());
        }
    }

    #[inline]
    pub fn set_uniform_mat4_f32(self: &mut Self, id: GLint, val: &[f32; 4 * 4]) {
        unsafe {
            gl::UniformMatrix4fv(id, 1, gl::FALSE, val.as_ptr());
        }
    }

    #[inline]
    pub fn get_attribute_hashmap(self: &Self) -> &HashMap<String, GLuint> {
        &self.attrib_ids
    }

    #[inline]
    pub fn get_uniform_hashmap(self: &Self) -> &HashMap<String, GLuint> {
        &self.uniform_ids
    }

    #[inline]
    pub fn get_uniform_id(self: &Self, name: &str) -> Option<GLuint> {
        self.uniform_ids.get(name).cloned()
    }

    #[inline]
    pub fn get_attribute_id(self: &Self, name: &str) -> Option<GLuint> {
        self.attrib_ids.get(name).cloned()
    }

    #[inline]
    pub fn get_sampler_id(self: &Self, name: &str) -> Option<GLuint> {
        self.get_uniform_id(name)
    }
}
