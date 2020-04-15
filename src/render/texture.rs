use crate::HasGLEnum;
use gl::types::*;
use std::convert::TryInto;
use crate::unwrap_result_or_ret;
use crate::unwrap_option_or_ret;

pub struct TextureBase {
    id: GLuint,
}

impl Drop for TextureBase {
    fn drop(self: &mut Self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

impl TextureBase {
    pub fn new(typ: GLenum) -> Self {
        let mut r = TextureBase { id: 0 };
        unsafe {
            gl::GenTextures(1, &mut r.id);
            // Need to set scale filter otherwise the texture can never be used so we set a reasonable default here to avoid errors
            gl::BindTexture(typ, r.id);
            gl::TexParameteri(
                typ,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR
                    .try_into()
                    .expect("FATAL Failure, faulty opengl implementation!"),
            );
            gl::TexParameteri(
                typ,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR
                    .try_into()
                    .expect("FATAL Failure, faulty opengl implementation!"),
            );
        }
        r
    }
}

// TODO: Make sure this works when using multiple textures
pub trait TextureFunc {
    fn bind_texture_for_sampling(self: &Self, sampler_id: GLuint) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(Self::get_type(), self.get_tex_base().id);
            gl::ActiveTexture(gl::TEXTURE0 + sampler_id);
        }
    }

    fn bind_texture_for_data(self: &Self) {
        unsafe {
            gl::BindTexture(Self::get_type(), self.get_tex_base().id);
        }
    }

    fn set_min_filter_of_bound_tex(self: &mut Self, min_filter: GLuint) {
        unsafe {
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_MIN_FILTER, min_filter.try_into().expect("FATAL Failure, faulty opengl implementation!"));
        }
    }

    fn set_mag_filter_of_bound_tex(self: &mut Self, mag_filter: GLuint) {
        unsafe {
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_MAG_FILTER, mag_filter.try_into().expect("FATAL Failure, faulty opengl implementation!"));
        }
    }

    fn set_x_wrap_of_bound_tex(self: &mut Self, wrap_x: GLint) {
        unsafe {
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_WRAP_S, wrap_x);
        }
    }

    fn set_y_wrap_of_bound_tex(self: &mut Self, wrap_y: GLint) {
        unsafe {
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_WRAP_T, wrap_y);
        }
    }

    fn set_z_wrap_of_bound_tex(self: &mut Self, wrap_z: GLint) {
        unsafe {
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_WRAP_R, wrap_z);
        }
    }

    fn get_type() -> GLenum;
    fn get_tex_base(self: &Self) -> &TextureBase;
}

pub struct Texture2D(TextureBase);
pub struct Texture2DArray(TextureBase);
pub struct Texture3D(TextureBase);

// IMPORTANT TODO: Add template to auto implement upload_data

impl Texture2D {
    pub fn new() -> Self {
        Self(TextureBase::new(Self::get_type()))
    }

    pub fn with_data<ET>(size: [usize; 2], data: &[ET], format: GLenum) -> Result<Self, String>
    where
        ET: HasGLEnum,
    {
        let mut r = Self::new();
        r.upload_data_to_bound_texture(size, data, format)?;
        Ok(r)
    }

    pub fn upload_data_to_bound_texture<ET>(
        self: &mut Self,
        size: [usize; 2],
        data: &[ET],
        format: GLenum,
    ) -> Result<(), String>
    where
        ET: HasGLEnum,
    {
        let l = data.len();
        let (internal_fmt, cpp) = unwrap_option_or_ret!(crate::format_to_gl_internal_format(
            (std::mem::size_of::<ET>() * 8).try_into().unwrap(),
            format,
        ), Err("Invalid format type!".to_owned()));

        if size[0] * size[1] * usize::from(cpp) != l {
            return Err(format!("Size provided is: {} pixels * {} pixels * {} values per pixel =/= {} (size of data array provided)!", size[0], size[1], cpp, size[0] * size[1] * usize::from(cpp)));
        }
        unsafe {
            gl::TexImage2D(
                Self::get_type(),
                0,
                internal_fmt,
                unwrap_result_or_ret!(size[0].try_into(), Err("Size[0] too big for opengl!".to_owned())),
                unwrap_result_or_ret!(size[1].try_into(), Err("Size[1] too big for opengl!".to_owned())),
                0,
                format,
                ET::get_gl_enum(),
                &data[0] as *const ET as *const std::ffi::c_void,
            );
        }
        Ok(())
    }
}

impl Texture2DArray {
    pub fn new() -> Self {
        Self(TextureBase::new(Self::get_type()))
    }

    pub fn upload_data_to_bound_texture<ET>(
        self: &mut Self,
        size: [usize; 3],
        data: &[ET],
        format: GLenum,
    ) -> Result<(), String>
    where
        ET: HasGLEnum,
    {
        let l = data.len();
        let (internal_fmt, cpp) = unwrap_option_or_ret!(crate::format_to_gl_internal_format(
            (std::mem::size_of::<ET>() * 8).try_into().unwrap(),
            format,
        ), Err("Invalid format type!".to_owned()));
        if size[0] * size[1] * size[2] * usize::from(cpp) != l {
            return Err(format!("Size provided is: {} pixels * {} pixels * {} images * {} values per pixel =/= {} (size of data array provided)!", size[0], size[1], size[2], cpp, size[0] * size[1] * size[2] * usize::from(cpp)));
        }
        unsafe {
            gl::TexImage3D(
                Self::get_type(),
                0,
                internal_fmt,
                unwrap_result_or_ret!(size[0].try_into(), Err("Size[0] provided is too big for opengl!".to_owned())),
                unwrap_result_or_ret!(size[1].try_into(), Err("Size[1] provided is too big for opengl!".to_owned())),
                unwrap_result_or_ret!(size[2].try_into(), Err("Size[2] provided is too big for opengl!".to_owned())),
                0,
                format,
                ET::get_gl_enum(),
                &data[0] as *const ET as *const std::ffi::c_void,
            );
        }
        Ok(())
    }
}

impl Texture3D {
    pub fn new() -> Self {
        Self(TextureBase::new(Self::get_type()))
    }

    pub fn upload_data_to_bound_texture<ET>(
        self: &mut Self,
        size: [usize; 3],
        data: &[ET],
        format: GLenum,
    ) -> Result<(), String>
    where
        ET: HasGLEnum,
    {
        let (internal_fmt, cpp) = unwrap_option_or_ret!(crate::format_to_gl_internal_format(
            (std::mem::size_of::<ET>() * 8).try_into().unwrap(),
            format,
        ), Err("Invalid format type!".to_owned()));
        if size[0] * size[1] * size[2] * usize::from(cpp) != data.len() {
            return Err(format!("Size provided is: {} pixels * {} pixels * {} pixels * {} values per pixel =/= {} (size of data array provided)!", size[0], size[1], size[2], cpp, size[0] * size[1] * size[2] * usize::from(cpp)));
        }
        unsafe {
            gl::TexImage3D(
                Self::get_type(),
                0,
                internal_fmt,
                unwrap_result_or_ret!(size[0].try_into(), Err("Size[0] provided is too big for opengl!".to_owned())),
                unwrap_result_or_ret!(size[1].try_into(), Err("Size[1] provided is too big for opengl!".to_owned())),
                unwrap_result_or_ret!(size[2].try_into(), Err("Size[2] provided is too big for opengl!".to_owned())),
                0,
                format,
                ET::get_gl_enum(),
                &data[0] as *const ET as *const std::ffi::c_void,
            );
        }
        Ok(())
    }
}

impl TextureFunc for Texture2D {
    #[inline]
    fn get_type() -> GLenum {
        gl::TEXTURE_2D
    }
    #[inline]
    fn get_tex_base(self: &Self) -> &TextureBase {
        &self.0
    }
}

impl TextureFunc for Texture2DArray {
    #[inline]
    fn get_type() -> GLenum {
        gl::TEXTURE_2D_ARRAY
    }
    #[inline]
    fn get_tex_base(self: &Self) -> &TextureBase {
        &self.0
    }
}

impl TextureFunc for Texture3D {
    #[inline]
    fn get_type() -> GLenum {
        gl::TEXTURE_3D
    }
    #[inline]
    fn get_tex_base(self: &Self) -> &TextureBase {
        &self.0
    }
}
