use crate::render::shader::*;
use crate::unwrap_or_ret_none;
use gl::types::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::CString;
use std::ptr;
use std::str;

pub struct Program {
    id: GLuint,
    /// TODO: Maybe optimise this as realistically there aren't going to be more than 100 entries
    /// in either of these hash maps so the 1 in O(1) becomes pretty big
    uniform_ids: HashMap<&'static str, GLuint>,
    attrib_ids: HashMap<&'static str, GLuint>,
}

impl Drop for Program {
    fn drop(self: &mut Self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

impl Program {
    pub fn new(shaders: &[&ShaderBase]) -> Result<Self, String> {
        let r = Program {
            id: unsafe { gl::CreateProgram() },
            uniform_ids: HashMap::new(),
            attrib_ids: HashMap::new(),
        };

        // Attach all shaders to program
        unsafe {
            shaders
                .iter()
                .for_each(|s| gl::AttachShader(r.id, s.get_id()));
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
        unsafe {
            shaders
                .iter()
                .for_each(|s| gl::DetachShader(r.id, s.get_id()));
        }

        Ok(r)
    }

    pub fn bind_program(self: &Self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    // General loader of values (private)
    /// TODO: Should probably get rid of inline(always) but i know this function is only used in like 2 spots and there is no real reason not to inline it
    #[inline(always)]
    fn get_id_of(
        self: &Self,
        name: &'static str,
        get_location: unsafe fn(GLuint, *const GLchar) -> GLint,
    ) -> Option<u32> {
        let id = {
            let cname = unwrap_or_ret_none!(CString::new(name.as_bytes()));
            unsafe { get_location(self.id, cname.as_ptr() as *const GLchar) }
        };
        if id < 0 {
            return None;
        }
        let id: u32 = unwrap_or_ret_none!(id.try_into());
        Some(id)
    }

    pub fn load_uniform(self: &mut Self, name: &'static str) -> Option<()> {
        // Check if already loaded, glGetUniformLocation can be pretty damn slow and a simple contains_key, especially on a hashbrown is probablly way faster
        if !self.uniform_ids.contains_key(name) {
            let u_id = self.get_id_of(name, gl::GetUniformLocation)?;
            self.uniform_ids.insert(name, u_id);
        }
        Some(())
    }

    pub fn load_sampler(self: &mut Self, name: &'static str) -> Option<()> {
        self.load_uniform(name)
    }

    pub fn load_attribute(self: &mut Self, name: &'static str) -> Option<()> {
        // Check if already loaded, glGetUniformLocation can be pretty damn slow and a simple contains_key, especially on a hashbrown is probably way faster
        if !self.attrib_ids.contains_key(name) {
            let a_id = self.get_id_of(name, gl::GetAttribLocation)?;
            self.attrib_ids.insert(name, a_id);
        }
        Some(())
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
    pub fn get_attribute_hashmap(self: &Self) -> &HashMap<&'static str, GLuint> {
        &self.attrib_ids
    }

    #[inline]
    pub fn get_uniform_hashmap(self: &Self) -> &HashMap<&'static str, GLuint> {
        &self.uniform_ids
    }

    /// I know unsafe is not a good idea but the code bloat is pretty big plus the only error that these functions can cause is either because memory corruption or bad usage of the api that could lead to undefined behaviour plus if the user really wants they can catch the unwrap soo, i'm sorry

    #[inline]
    pub fn get_uniform_id(self: &Self, name: &'static str) -> GLuint {
        *self.uniform_ids.get(name).unwrap()
    }

    #[inline]
    pub fn get_attribute_id(self: &Self, name: &'static str) -> GLuint {
        *self.attrib_ids.get(name).unwrap()
    }

    #[inline]
    pub fn get_sampler_id(self: &Self, name: &'static str) -> GLuint {
        self.get_uniform_id(name)
    }
}
