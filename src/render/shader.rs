use gl::types::*;
use std::convert::TryInto;
use std::ffi::CString;
use std::ptr;
use std::str;

pub struct ShaderBase {
    id: GLuint,
}

impl Drop for ShaderBase {
    fn drop(self: &mut Self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

impl ShaderBase {
    /// WARNING: Pretty slow if error occurs
    /// NOTE: Just in general shader compilation is slow so use new only when you need to
    pub fn new(src: &str, sh_type: GLenum) -> Result<Self, String> {
        let r = ShaderBase {
            id: unsafe { gl::CreateShader(sh_type) },
        };
        unsafe {
            {
                let csrc = match CString::new(src.as_bytes()) {
                    Ok(val) => val,
                    Err(_) => return Err(String::from("Failed to convert string to CString!")),
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
				Err(_) => return Err(String::from("Length of error message of shader compilation is either too big or negative!")),
			    };
                let mut buf = Vec::<u8>::with_capacity(new_len);
                buf.set_len(new_len - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(r.id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);

                return match str::from_utf8(&buf) {
                    Ok(msg) => {
                        let t = String::from(
                            crate::shader_glenum_to_string(sh_type).unwrap_or("unknown"),
                        );
                        Err(String::from(msg)
                            + &String::from("In shader of type: ")
                            + &t
                            + &String::from("!\n"))
                    }

                    Err(std::str::Utf8Error { .. }) => {
                        Err(String::from("ShaderInfoLog not a valid utf8 string!"))
                    }
                };
            }
        }
        Ok(r)
    }

    // NEEDED BY Program
    pub(crate) fn get_id(self: &Self) -> GLuint {
        self.id.clone() // make sure we don't give up our id ( i know this is redundant )
    }
}

pub struct VertexShader(ShaderBase);
pub struct FragmentShader(ShaderBase);

impl VertexShader {
    /// This just runs Shader::new to take a look at that
    pub fn new(src: &str) -> Result<Self, String> {
        Ok(VertexShader(ShaderBase::new(src, gl::VERTEX_SHADER)?))
    }

    pub fn get_shader_base(self: &Self) -> &ShaderBase {
        &self.0
    }
}

impl Into<ShaderBase> for VertexShader {
    // Consume VertexShader and pass ownership of it's only value as output
    fn into(self: Self) -> ShaderBase {
        self.0
    }
}

impl FragmentShader {
    /// This just runs Shader::new to take a look at that
    pub fn new(src: &str) -> Result<Self, String> {
        Ok(FragmentShader(ShaderBase::new(src, gl::FRAGMENT_SHADER)?))
    }
    pub fn get_shader_base(self: &Self) -> &ShaderBase {
        &self.0
    }
}

impl Into<ShaderBase> for FragmentShader {
    // Consume VertexShader and pass ownership of it's only value as output
    fn into(self: Self) -> ShaderBase {
        self.0
    }
}
