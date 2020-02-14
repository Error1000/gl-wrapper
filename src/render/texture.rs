use crate::unwrap_or_ret_none;
use gl::types::*;
use std::convert::TryInto;

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

pub trait TextureFunc {
    fn bind_texture_for_sampling(self: &Self, sampler_id: GLuint) {
        unsafe {
            self.bind_texture_for_data();
            gl::ActiveTexture(gl::TEXTURE0 + sampler_id);
        }
    }

    fn bind_texture_for_data(self: &Self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(Self::get_type(), self.get_tex_base().id);
        }
    }

    fn set_min_filter_of_bound_tex(self: &mut Self, min_filter: GLint) {
        unsafe {
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_MIN_FILTER, min_filter);
        }
    }

    fn set_mag_filter_of_bound_tex(self: &mut Self, mag_filter: GLint) {
        unsafe {
            gl::TexParameteri(Self::get_type(), gl::TEXTURE_MAG_FILTER, mag_filter);
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

    pub fn upload_data_to_bound_texture<ET: 'static>(
        self: &mut Self,
        size: [u32; 2],
        data: &[ET],
        format: GLenum
    ) -> Option<()> {
        let l: u32 = unwrap_or_ret_none!(data.len().try_into());
        let (internal_fmt, cpp) = crate::format_to_gl_internal_format(
            (std::mem::size_of::<ET>() * 8).try_into().unwrap(),
            format
        )?;
        if size[0] * size[1] * u32::from(cpp) != l {
            return None;
        }
        unsafe {
            gl::TexImage2D(
                Self::get_type(),
                0,
                internal_fmt,
                unwrap_or_ret_none!(size[0].try_into()),
                unwrap_or_ret_none!(size[1].try_into()),
                0,
                format,
                crate::type_to_gl_enum::<ET>()?,
                &data[0] as *const ET as *const std::ffi::c_void,
            );
        }
        Some(())
    }
}

impl Texture2DArray {
    pub fn new() -> Self {
        Self(TextureBase::new(Self::get_type()))
    }

    pub fn upload_data_to_bound_texture<ET: 'static>(
        self: &mut Self,
        size: [u32; 3],
        data: &[ET],
        format: GLenum
    ) -> Option<()> {
        let l: u32 = unwrap_or_ret_none!(data.len().try_into());
        let (internal_fmt, cpp) = crate::format_to_gl_internal_format(
            (std::mem::size_of::<ET>() * 8).try_into().unwrap(),
            format
        )?;
        if size[0] * size[1] * size[2] * u32::from(cpp) != l {
            return None;
        }
        unsafe {
            gl::TexImage3D(
                Self::get_type(),
                0,
                internal_fmt,
                unwrap_or_ret_none!(size[0].try_into()),
                unwrap_or_ret_none!(size[1].try_into()),
                unwrap_or_ret_none!(size[2].try_into()),
                0,
                format,
                crate::type_to_gl_enum::<ET>()?,
                &data[0] as *const ET as *const std::ffi::c_void,
            );
        }
        Some(())
    }
}

impl Texture3D {
    pub fn new() -> Self {
        Self(TextureBase::new(Self::get_type()))
    }

    pub fn upload_data_to_bound_texture<ET: 'static>(
        self: &mut Self,
        size: [u32; 3],
        data: &[ET],
        format: GLenum
    ) -> Option<()> {
        let l: u32 = unwrap_or_ret_none!(data.len().try_into());
        let (internal_fmt, cpp) = crate::format_to_gl_internal_format(
            (std::mem::size_of::<ET>() * 8).try_into().unwrap(),
            format
        )?;
        if size[0] * size[1] * size[2] * u32::from(cpp) != l {
            return None;
        }
        unsafe {
            gl::TexImage3D(
                Self::get_type(),
                0,
                internal_fmt,
                unwrap_or_ret_none!(size[0].try_into()),
                unwrap_or_ret_none!(size[1].try_into()),
                unwrap_or_ret_none!(size[2].try_into()),
                0,
                format,
                crate::type_to_gl_enum::<ET>()?,
                &data[0] as *const ET as *const std::ffi::c_void,
            );
        }
        Some(())
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
