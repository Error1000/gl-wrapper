use gl::types::*;
use std::convert::TryInto;
use crate::api;

pub struct TextureBase{
    id: GLuint
}

impl Drop for TextureBase{
    fn drop(self: &mut Self){
        unsafe{ gl::DeleteTextures(1, &self.id); }
    }
}

impl TextureBase{
    fn new(typ: GLenum) -> Self{
        let mut r = TextureBase{
            id: 0
        };
        unsafe{
            gl::GenTextures(1, &mut r.id);
            // Need to set scale filter otherwise the texture can never be used so we set a reasonable default here to avoid errors 
            gl::BindTexture(typ, r.id);
            gl::TexParameteri(typ, gl::TEXTURE_MIN_FILTER, gl::LINEAR.try_into().expect("FATAL Failure, faulty opengl implementation!"));
            gl::TexParameteri(typ, gl::TEXTURE_MAG_FILTER, gl::LINEAR.try_into().expect("FATAL Failure, faulty opengl implementation!"));
        }
        r
    }
}


pub trait TextureFunc{
    fn bind_texture_for_sampling(self: &Self, sampler_id: GLuint){
            unsafe{
                self.bind_texture_for_data();
                gl::ActiveTexture(gl::TEXTURE0+sampler_id);
            }
    }

    fn bind_texture_for_data(self: &Self){
        unsafe{
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(Self::get_type(), self.get_base_tex().id);
        }
    }

    fn set_min_filter_of_bound_tex(self: &mut Self, min_filter: GLint){
        unsafe{
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_MIN_FILTER, min_filter);
        }
    }

    fn set_mag_filter_of_bound_tex(self: &mut Self, mag_filter: GLint){
        unsafe{
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_MAG_FILTER, mag_filter);
        }
    }

    fn set_x_wrap_of_bound_tex(self: &mut Self, wrap_x: GLint){
        unsafe{
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_WRAP_S, wrap_x);
        }
    }

    fn set_y_wrap_of_bound_tex(self: &mut Self, wrap_y: GLint){
        unsafe{
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_WRAP_T, wrap_y);
        }
    }

    fn set_z_wrap_of_bound_tex(self: &mut Self, wrap_z: GLint){
        unsafe{
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_WRAP_R, wrap_z);
        }
    }

    fn get_type() -> GLenum;
    fn get_base_tex(self: &Self) -> &TextureBase;
}



pub struct Texture2D(TextureBase);
pub struct Texture2DArray(TextureBase);
pub struct Texture3D(TextureBase);



impl Texture2D{
    pub fn new() -> Self{
        Self(TextureBase::new(Self::get_type()))
    }

    /// CPP = channels per pixel ( examples: RGBA = 4 channels, RGB = 3 channels, RG = 2 channels, R = 1 channel, ... )
    pub fn upload_data_to_bound_texture<ET>(self: &mut Self, size: [GLint; 2], data: &[ET], cpp: u8)-> Option<()> where ET: 'static{
        let l: i32 = match data.len().try_into(){
            Ok(v) => v,
            Err(_) => return None,
        };
        if size[0] * size[1] * i32::from(cpp) != l { return None; }
        let (internal_fmt, fmt) = api::texture_bits_to_opengl_types((std::mem::size_of::<ET>()*8).try_into().unwrap(), cpp)?;
        unsafe{
            gl::TexImage2D(Self::get_type(), 0, internal_fmt, size[0], size[1], 0, fmt, api::type_to_gl_enum::<ET>()?, &data[0] as *const ET as *const std::ffi::c_void);
        }
        Some(())
    }
}

impl Texture2DArray{
    pub fn new() -> Self{
        Self(TextureBase::new(Self::get_type()))
    }

    /// CPP = channels per pixel ( examples: RGBA = 4 channels, RGB = 3 channels, RG = 2 channels, R = 1 channel, ... )
    pub fn upload_data_to_bound_texture<ET>(self: &mut Self, size: [GLint; 3], data: &[ET], cpp: u8)-> Option<()> where ET: 'static{
        let l: i32 = match data.len().try_into(){
            Ok(v) => v,
            Err(_) => return None,
        };
        if size[0] * size[1] * size[2] * i32::from(cpp) != l { return None; }
        let (internal_fmt, fmt) = api::texture_bits_to_opengl_types((std::mem::size_of::<ET>()*8).try_into().unwrap(), cpp)?;
        unsafe{
            gl::TexImage3D(Self::get_type(), 0, internal_fmt, size[0], size[1], size[2], 0, fmt, api::type_to_gl_enum::<ET>()?, &data[0] as *const ET as *const std::ffi::c_void);
        }
        Some(())
    }
}

impl Texture3D{
    pub fn new() -> Self{
        Self(TextureBase::new(Self::get_type()))
    }

    /// CPP = channels per pixel ( examples: RGBA = 4 channels, RGB = 3 channels, RG = 2 channels, R = 1 channel, ... )
    pub fn upload_data_to_bound_texture<ET>(self: &mut Self, size: [GLint; 3], data: &[ET], cpp: u8)-> Option<()> where ET: 'static{
        let l: i32 = match data.len().try_into(){
            Ok(v) => v,
            Err(_) => return None,
        };
        if size[0] * size[1] * size[2] * i32::from(cpp) != l { return None; }
        let (internal_fmt, fmt) = api::texture_bits_to_opengl_types((std::mem::size_of::<ET>()*8).try_into().unwrap(), cpp)?;
        unsafe{
            gl::TexImage3D(Self::get_type(), 0, internal_fmt, size[0], size[1], size[2], 0, fmt, api::type_to_gl_enum::<ET>()?, &data[0] as *const ET as *const std::ffi::c_void);
        }
        Some(())
    }
}



impl TextureFunc for Texture2D{
    #[inline]
    fn get_type() -> GLenum { gl::TEXTURE_2D }
    #[inline]
    fn get_base_tex(self: &Self) -> &TextureBase { &self.0 }

}

impl TextureFunc for Texture2DArray{
    #[inline]
    fn get_type() -> GLenum { gl::TEXTURE_2D_ARRAY }
    #[inline]
    fn get_base_tex(self: &Self) -> &TextureBase { &self.0 }
}

impl TextureFunc for Texture3D{
    #[inline]
    fn get_type() -> GLenum { gl::TEXTURE_3D }
    #[inline]
    fn get_base_tex(self: &Self) -> &TextureBase { &self.0 }
}