use crate::render::shader::*;
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
    attribute_ids: HashMap<&'static str, GLuint>,
}

impl Drop for Program {
    fn drop(self: &mut Self) {
        unsafe { gl::DeleteProgram(self.id); }
    }
}

impl Program {
    pub fn new(shaders: &[&Shader]) -> Result<Self, String> {
        let r = Program {
                id: unsafe{ gl::CreateProgram() },
                uniform_ids: HashMap::new(),
                attribute_ids: HashMap::new(),
        };

        // Attach all shaders to program
        unsafe {
            shaders
                .iter()
                .for_each(|s| gl::AttachShader(r.id, s.get_id()));
            gl::LinkProgram(r.id);
        }

        let mut program_compiled: GLint = gl::FALSE.into();
        unsafe { gl::GetProgramiv(r.id, gl::LINK_STATUS, &mut program_compiled); }
        if program_compiled != gl::TRUE.into() {
            let mut len: i32 = 0;
            unsafe { gl::GetProgramiv(r.id, gl::INFO_LOG_LENGTH, &mut len); }
            let new_len: usize = match len.try_into() {
                Ok(val) => val,
                Err(_) => return Err(String::from("")),
            };
            let mut buf = Vec::<u8>::with_capacity(new_len);
            unsafe {
                gl::GetProgramInfoLog(
                    r.id,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
            }
            match str::from_utf8(&buf) {
                Ok(msg) =>
                    return Err(String::from(msg)),
                Err(std::str::Utf8Error { .. }) =>
                    return Err(String::from("ProgramInfoLog not a valid utf8 string")),
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
        unsafe { gl::UseProgram(self.id); }
    }
    // General loader of values
    #[inline]
    fn get_id_of(self: &Self, name: &'static str, get_location: unsafe fn(GLuint, *const GLchar) -> GLint) -> Option<u32>{
        let id = {
            let cname = match CString::new(name.as_bytes()){
                Ok(val) => val,
                Err(_) => return None
            };
            unsafe { get_location(self.id, cname.as_ptr() as *const GLchar) }
        };
        if id < 0 { return None; }
        let id: u32 = match id.try_into(){
            Ok(val) => val,
            Err(_) => return None
        };
        Some(id)
    }

    pub fn load_uniform(self: &mut Self, name: &'static str) -> Option<()> {
        // Check if already loaded, glGetUniformLocation can be pretty damn slow and a simple contains_key, especially on a hashbrown is probablly way faster
        if !self.uniform_ids.contains_key(name) {
            let u_id = self.get_id_of(name,gl::GetUniformLocation)?;
            self.uniform_ids.insert(name, u_id);
        }
        Some(())
    }

    pub fn load_sampler(self: &mut Self, name: &'static str) -> Option<()> {
        if !self.uniform_ids.contains_key(name){
            let s_id = self.get_id_of(name, gl::GetUniformLocation)?;
            self.uniform_ids.insert(name, s_id);
        }
        Some(())
    }

    pub fn load_attribute(self: &mut Self, name: &'static str) -> Option<()> {
        // Check if already loaded, glGetUniformLocation can be pretty damn slow and a simple contains_key, especially on a hashbrown is probably way faster
        if !self.attribute_ids.contains_key(name) {
            let a_id = self.get_id_of(name, gl::GetAttribLocation)?;
            self.attribute_ids.insert(name, a_id);
        }
        Some(())
    }

    pub fn clear_all_loaded(self: &mut Self) {
        self.uniform_ids.clear();
        self.attribute_ids.clear();
    }

    pub fn set_uniform_i32(self: &mut Self, id: GLint, val: i32) {
        unsafe { gl::Uniform1i(id, val); }
    }
    pub fn set_uniform_u32(self: &mut Self, id: GLint, val: u32) {
        unsafe { gl::Uniform1ui(id, val); }
    }
    pub fn st_uniform_f32(self: &mut Self, id: GLint, val: f32) {
        unsafe { gl::Uniform1f(id, val); }
    }

    pub fn get_uniform_id(self: &Self, name: &'static str) -> Option<GLuint> {
        Some(*(self.uniform_ids.get(name)?))
    }
    pub fn get_attribute_id(self: &Self, name: &'static str) -> Option<GLuint> {
        Some(*(self.attribute_ids.get(name)?))
    }
    pub fn get_sampler_id(self: &Self, name: &'static str) -> Option<GLuint> {
        self.get_uniform_id(name)
    }

    pub fn get_attribute_hashmap(self: &Self) -> &HashMap<&'static str, GLuint>{ &self.attribute_ids }
    pub fn get_uniform_hashmap(self: &Self) -> &HashMap<&'static str, GLuint>{ &self.uniform_ids }

}