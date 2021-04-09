use crate::unwrap_option_or_ret;
use crate::unwrap_result_or_ret;
use crate::HasGLEnum;
use gl::types::*;
use std::{convert::TryInto};


#[inline(always)]
unsafe fn internal_gl_tex_image<const N: usize>(target: GLenum, level: GLint, internal_format: GLint, dim: &[GLsizei; N], border: GLint, format: GLenum, typ: GLenum, data:  *const GLvoid){
    match N{
        2 =>
            { gl::TexImage2D(target, level, internal_format, dim[0], dim[1], border, format, typ, data) },
        3  =>
            { gl::TexImage3D(target, level, internal_format, dim[0], dim[1], dim[2], border, format, typ, data) },
        _  => panic!("Unspported dimensions for texture!")
    } 
}

/*
    WARNING: The generics this struct takes could lead to unsound code,
    since generic contraints have not landed yet, and generics are still not mature enough
    to allow me to remove the hole for now this is public, however either way you should
    use the predefined types Texture2D, Texture2DArr or Texture3D instead
*/
pub struct TextureBase<const N: usize, const TYP: GLenum> {
    id: GLuint
}

pub type Texture2D = TextureBase<2, {gl::TEXTURE_2D}>;
pub type Texture2DArr = TextureBase<3, {gl::TEXTURE_2D_ARRAY}>;
pub type Texture3D = TextureBase<3, {gl::TEXTURE_3D}>;

impl<const N: usize, const TYP: GLenum> TextureBase<N, TYP>{
    pub fn bind_texture_for_sampling(self: &Self, sampler_id: GLuint) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(TYP, self.id);
            gl::ActiveTexture(gl::TEXTURE0 + sampler_id);
        }
    }

    pub fn bind_texture_for_data(self: &Self) {
        unsafe {
            gl::BindTexture(TYP, self.id);
        }
    }


}

impl<const N: usize, const TYP: GLenum> TextureFunc<N, TYP> for TextureBase<N, TYP>{ 
    fn new() -> Self {
        let mut r = TextureBase { id: 0 };
        unsafe {
            gl::GenTextures(1, &mut r.id);
            // Need to set scale filter otherwise the texture can't be used so we set a reasonable default here to avoid errors
            gl::BindTexture(TYP, r.id);
            gl::TexParameteri(
                TYP,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR.try_into().expect("Converting to gl types!"),
            );
            gl::TexParameteri(
                TYP,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR.try_into().expect("Converting to gl types!"),
            );
        }
        r
    }
}

impl<const N: usize, const TYP: GLenum> Drop for TextureBase<N, TYP> {
    fn drop(self: &mut Self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

// TODO: Make sure this works when using multiple textures
pub trait TextureFunc<const N: usize, const TYP: GLenum> 
where Self: Sized{

    #[inline(always)]
    fn set_min_filter_of_bound_tex(self: &mut Self, min_filter: GLuint) {
        unsafe {
            gl::TexParameteri(
                TYP,
                gl::TEXTURE_MIN_FILTER,
                min_filter.try_into().expect("Converting to gl types!"),
            );
        }
    }
 
    #[inline(always)]
    fn set_mag_filter_of_bound_tex(self: &mut Self, mag_filter: GLuint) {
        unsafe {
            gl::TexParameteri(
                TYP,
                gl::TEXTURE_MAG_FILTER,
                mag_filter.try_into().expect("Converting to gl types!"),
            );
        }
    }

    #[inline(always)]
    fn set_x_wrap_of_bound_tex(self: &mut Self, wrap_x: GLint) {
        unsafe {
            gl::TexParameteri(TYP, gl::TEXTURE_WRAP_S, wrap_x);
        }
    }

    #[inline(always)]
    fn set_y_wrap_of_bound_tex(self: &mut Self, wrap_y: GLint) {
        unsafe {
            gl::TexParameteri(TYP, gl::TEXTURE_WRAP_T, wrap_y);
        }
    }

    #[inline(always)]
    fn set_z_wrap_of_bound_tex(self: &mut Self, wrap_z: GLint) {
        unsafe {
            gl::TexParameteri(TYP, gl::TEXTURE_WRAP_R, wrap_z);
        }
    }


    fn upload_data_to_bound_texture<ET>(
        self: &mut Self,
        size: [usize; N],
        data: &[ET],
        format: GLenum,
    ) -> Result<(), String>
    where
        ET: HasGLEnum{
    
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
        for i in 0..N{
            formatted_siz[i] = unwrap_result_or_ret!(size[i].try_into(), Err(format!("Size[{}] malformed!", i)));
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
 
    fn with_data<ET>(size: [usize; N], data: &[ET], format: GLenum) -> Result<Self, String>
    where ET: HasGLEnum{
        let mut r = Self::new();
        r.upload_data_to_bound_texture(size, data, format)?;
        Ok(r)
    }

    fn new() -> Self;
}

