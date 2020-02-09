use crate::api;
use gl::types::*;
use std::convert::TryInto;
use std::ffi::CString;
use std::ptr;
use std::str;

pub struct Shader {
    id: GLuint
}

impl Drop for Shader {
    fn drop(self: &mut Self) {
        unsafe { gl::DeleteShader(self.id); }
    }
}

impl Shader {
    /// WARNING: Pretty slow if error occurs
    /// NOTE: Just in general shader compilation is slow so use new only when you need to
    pub fn new(src: &str, sh_type: GLenum) -> Result<Self, String> {
        let r = Shader {
                id: unsafe{ gl::CreateShader(sh_type) },
        };
        unsafe {
            {
                let csrc = match CString::new(src.as_bytes()){
                    Ok(val) => val,
                    Err(_) => return Err(String::from("Failed to convert string to CString!"))
                };
                gl::ShaderSource(r.id, 1, &csrc.as_ptr(), ptr::null());
            }
            gl::CompileShader(r.id);

            let mut shader_compiled: GLint = gl::FALSE.into();
            gl::GetShaderiv(r.id, gl::COMPILE_STATUS, &mut shader_compiled);

            // Fail on error
            if shader_compiled != gl::TRUE.into() {
                let mut len: i32 = 0;
                gl::GetShaderiv(r.id, gl::INFO_LOG_LENGTH, &mut len);
                let new_len: usize = match len.try_into(){
				Ok(val) => val,
				Err(_) => return Err(String::from("Length of error message of shader compialtion is either too big or negative!")),
			    };
                let mut buf = Vec::<u8>::with_capacity(new_len);
                buf.set_len(new_len - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(
                    r.id,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );

                return match str::from_utf8(&buf) {
                    Ok(msg) => {
                        let t = String::from(match api::gl_shader_enum_to_string(sh_type) {
                            Some(s) => s,
                            None => "unknown",
                        });
                        Err(String::from(msg)
                            + &String::from("In shader of type: ")
                            + &t
                            + &String::from("!\n"))
                    },

                    Err(std::str::Utf8Error { .. }) =>
                        Err(String::from("ShaderInfoLog not a valid utf8 string!"))
                };

            }
        }
        Ok(r)
    }
  
   // NEEDED BY Program
   pub(crate) fn get_id(self: &Self) -> GLuint {
       self.id
   }
}




pub struct VertexShader(Shader);
pub struct FragmentShader(Shader);

impl VertexShader {
    pub fn new(src: &str) -> Result<Self, String> {
        Ok(VertexShader(Shader::new(src, gl::VERTEX_SHADER)?))
    }
}

impl Into<Shader> for VertexShader{
    // Consume VertexShader and pass ownership of it's only value as output
    fn into(self: Self) -> Shader{ self.0 }
}

impl FragmentShader {
    pub fn new(src: &str) -> Result<Self, String> {
        Ok(FragmentShader(Shader::new(src, gl::FRAGMENT_SHADER)?))
    }
}

impl Into<Shader> for FragmentShader{
    // Consume VertexShader and pass ownership of it's only value as output
    fn into(self: Self) -> Shader{ self.0 }
}