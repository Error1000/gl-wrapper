use crate::unwrap_option_or_ret;
use crate::unwrap_result_or_ret;
use crate::HasGLEnum;
use gl::types::*;
use std::convert::TryInto;


#[inline(always)]
unsafe fn internal_gl_tex_image<const N: usize>(
    target: GLenum,
    level: GLint,
    internal_format: GLint,
    dim: &[GLsizei; N],
    border: GLint,
    format: GLenum,
    typ: GLenum,
    data: *const GLvoid,
) {
    match N {
        2 => gl::TexImage2D(
            target,
            level,
            internal_format,
            dim[0],
            dim[1],
            border,
            format,
            typ,
            data,
        ),
        3 => gl::TexImage3D(
            target,
            level,
            internal_format,
            dim[0],
            dim[1],
            dim[2],
            border,
            format,
            typ,
            data,
        ),
        _ => panic!("Unspported dimensions for texture!"),
    }
}

mod texture_base{
use super::*;
use gl::types::{GLenum, GLuint};
use one_user::one_user;


impl<const N: usize, const TYP: GLenum> texturebase_binder::OnBind for TextureBase<N, TYP>{
    #[inline(always)]
    fn on_bind<const SLOT: usize>(&self) {
        if SLOT != (*texturebase_binder::LAST_SLOT).load(core::sync::atomic::Ordering::SeqCst){
                unsafe{
                    gl::ActiveTexture(gl::TEXTURE0 + SLOT as u32);
                }
        }
        unsafe{ gl::BindTexture(TYP, self.id); }

    }
}


#[one_user(256)]
pub struct TextureBase<const N: usize, const TYP: GLenum> {
    id: GLuint,
}


impl<const N: usize, const TYP: GLenum> TextureBase<N, TYP> {
    pub fn new<const BI: usize>(bn: &mut texturebase_binder::BOUNCER<BI>) -> UnboundTexture<N, TYP> {
        let mut r = TextureBase { id: 0 };
        unsafe {
            gl::GenTextures(1, &mut r.id);
        }
        let mut r = UnboundTexture::from(r);
        {
            // Need to set min and mag filter because opengl by default uses mipmaps and we currently do not support that
            // Not setting the min and mag filters would probablly mean textures would appear as blank
            let mut r = r.bind_mut(bn);
            r.set_mag_filter_of_bound_tex(gl::LINEAR);
            r.set_min_filter_of_bound_tex(gl::LINEAR);
        }
        r
    }

    #[inline(always)]
    pub fn set_min_filter_of_bound_tex(self: &mut Self, min_filter: GLuint) {
        unsafe {
            gl::TexParameteri(
                TYP,
                gl::TEXTURE_MIN_FILTER,
                min_filter.try_into().expect("Converting to gl types!"),
            );
        }
    }

    #[inline(always)]
    pub fn set_mag_filter_of_bound_tex(self: &mut Self, mag_filter: GLuint) {
        unsafe {
            gl::TexParameteri(
                TYP,
                gl::TEXTURE_MAG_FILTER,
                mag_filter.try_into().expect("Converting to gl types!"),
            );
        }
    }

    #[inline(always)]
    pub fn set_x_wrap_of_bound_tex(self: &mut Self, wrap_x: GLint) {
        unsafe {
            gl::TexParameteri(TYP, gl::TEXTURE_WRAP_S, wrap_x);
        }
    }

    #[inline(always)]
    pub fn set_y_wrap_of_bound_tex(self: &mut Self, wrap_y: GLint) {
        unsafe {
            gl::TexParameteri(TYP, gl::TEXTURE_WRAP_T, wrap_y);
        }
    }

    #[inline(always)]
    pub fn set_z_wrap_of_bound_tex(self: &mut Self, wrap_z: GLint) {
        unsafe {
            gl::TexParameteri(TYP, gl::TEXTURE_WRAP_R, wrap_z);
        }
    }

    pub fn upload_data_to_texture<ET>(
        self: &mut Self,
        size: [usize; N],
        data: &[ET],
        format: GLenum,
    ) -> Result<(), String>
    where
        ET: HasGLEnum,
    {
        let l = data.len();
        let (internal_fmt, cpp) = unwrap_option_or_ret!(
            crate::format_to_gl_internal_format(
                (std::mem::size_of::<ET>() * 8).try_into().unwrap(),
                format,
            ),
            Err("Invalid format type!".to_owned())
        );

        if size[0] * size[1] * usize::from(cpp) != l {
            return Err(format!("Size provided is: {} pixels * {} pixels * {} values per pixel =/= {} (size of data array provided)!", size[0], size[1], cpp, size[0] * size[1] * usize::from(cpp)));
        }
        let mut formatted_siz: [GLsizei; N] = [0; N];
        for i in 0..N {
            formatted_siz[i] =
                unwrap_result_or_ret!(size[i].try_into(), Err(format!("Size[{}] malformed!", i)));
        }

        unsafe {
            internal_gl_tex_image::<N>(
                TYP,
                0,
                internal_fmt,
                &formatted_siz,
                0,
                format,
                ET::get_gl_type(),
                &data[0] as *const ET as *const std::ffi::c_void,
            );
        }
        Ok(())
    }

    pub fn with_data<ET, const BI: usize>(
        bn: &mut TextureBouncer<BI>,
        size: [usize; N],
        data: &[ET],
        format: GLenum,
    ) -> Result<UnboundTexture<N, TYP>, String>
    where
        ET: HasGLEnum,
    {
        let mut r = Self::new(bn);
        {
            let mut r = r.bind_mut(bn);
            r.upload_data_to_texture(size, data, format)?;
        }
        Ok(r)
    }
}

impl<const N: usize, const TYP: GLenum> Drop for TextureBase<N, TYP> {
    fn drop(self: &mut Self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}


}

pub type Texture2D = texture_base::TextureBase<2, { gl::TEXTURE_2D }>;
pub type Texture2DArr = texture_base::TextureBase<3, { gl::TEXTURE_2D_ARRAY }>;
pub type Texture3D = texture_base::TextureBase<3, { gl::TEXTURE_3D }>;

pub type TextureBouncer<const BI: usize> = texture_base::TextureBaseBouncer<BI>;
pub type UnboundTexture<const N: usize, const TYP: GLenum> = texture_base::UnboundTextureBase<N, TYP>;
pub type BoundTexture<'a, const BI: usize, const N: usize, const TYP: GLenum> = texture_base::BoundTextureBase<'a, N, TYP, BI>;
