use crate::unwrap_option_or_ret;
use crate::unwrap_result_or_ret;
use crate::HasGLEnum;
use gl::types::*;
use std::convert::TryInto;

use self::tex_binder::OnBind;

pub type TextureBouncer<const BI: usize> = tex_binder::BOUNCER<BI>;
pub type UnboundTexture<const N: usize, const TYP: GLenum> = tex_binder::Unbound<N, TYP>;
pub type BoundProgram<'a, const BI: usize, const N: usize, const TYP: GLenum> = tex_binder::Bound<'a, BI, N, TYP>;

mod tex_binder {
    const NBOUNCERS: usize = 256;
    type Usable<const N: usize, const TYP: GLenum> = super::texture_base::TextureBase<N, TYP>;

    pub trait OnBind {
        fn on_bind<const BI: usize>(&mut self);
    }

    use bitvec::prelude::*;
    use gl::types::GLenum;
    use std::{
        ops::{Deref, DerefMut},
        sync::{atomic::AtomicUsize, Mutex},
    };

    lazy_static! {
        static ref BOUNCER_GUARD: Mutex<BitArr!(for NBOUNCERS, in Msb0, u8)> = Mutex::new(BitArray::zeroed()); // BOUNCER_GUARD is private, this is important because we don't want somebody take()-ing the intialised OnceCell, leaving it uninitialised, and being able to call new() again on BOUNCER again and have two BOUNCERs
        /// SAFTEY: LAST_BOUND is unreliable, don't rely on it for correctness
        pub static ref LAST_BOUND: AtomicUsize = AtomicUsize::new(0);
    }

    pub struct BOUNCER<const BI: usize>(()); // NOTE: () is private, this is important so that the only way to get a BOUNCER instance is to use new()

    impl<const BI: usize> BOUNCER<BI> {
        /// IMPORTANT: Only one bouncer can exist ever
        /// SAFETY: We are the only ones who can access BOUNCER_GUARD because it is private and we can use that to make sure that we only create one BOUNCER ever
        #[inline]
        pub fn new() -> Option<Self> {
            if BI >= NBOUNCERS {
                return None;
            }
            let mut lck = BOUNCER_GUARD.try_lock().ok()?;
            if lck.get(BI).unwrap() == false { 
                lck.set(BI, true);
                Some(BOUNCER(()))
            } else {
                None
            }
        }
    }

    // Because there only ever exists one bouncer a &mut to a BOUNCER must be unique, so thre can only ever exist one Bound
    pub struct Bound<'a, const BI: usize, const N: usize, const TYP: GLenum>(
        &'a mut Usable<N, TYP>,
        &'a mut BOUNCER<BI>,
    );

    impl<const BI: usize, const N: usize, const TYP: GLenum> Deref for Bound<'_, BI, N, TYP> {
        type Target = Usable<N, TYP>;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<const BI: usize, const N: usize, const TYP: GLenum> DerefMut for Bound<'_, BI, N, TYP> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    pub struct Unbound<const N: usize, const TYP: GLenum>(Usable<N, TYP>)
    where
        Usable<N, TYP>: OnBind; // Usable is private, this is important because it means to get a Usable you must go through bind which goes through a Bound which requires a &mut BOUNCER, whichs is unique, so no matter how many Unbound there are, there will only ever be one Bound at a time

    impl<const N: usize, const TYP: GLenum> Unbound<N, TYP> {
        #[inline]
        pub fn from(val: Usable<N, TYP>) -> Unbound<N, TYP> {
            Unbound(val)
        } // Takes a Usable and makes it an Unbound, this is fine since Usable can control how it's constructed and return an Unbound(Usable) instead of a Usable so there is no way a normal user can make a Usable without it being Unbound
        #[inline]
        pub fn bind<'a, const BI: usize>(
            &'a mut self,
            bn: &'a mut BOUNCER<BI>,
        ) -> Bound<'a, BI, N, TYP> {
            self.0.on_bind::<BI>();
            LAST_BOUND.store(BI, core::sync::atomic::Ordering::Relaxed);
            Bound(&mut self.0, bn)
        }
    }
}

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



impl<const N: usize, const TYP: GLenum> OnBind for texture_base::TextureBase<N, TYP> {
    #[inline(always)]
    fn on_bind<const BI: usize>(&mut self) {
        if BI != (*tex_binder::LAST_BOUND).load(core::sync::atomic::Ordering::Relaxed){
                unsafe{
                    gl::ActiveTexture(gl::TEXTURE0 + BI as u32);
                }
        }
        unsafe{ gl::BindTexture(TYP, self.id); }
    }
}


pub struct TextureBase<const N: usize, const TYP: GLenum> {
    id: GLuint,
}


impl<const N: usize, const TYP: GLenum> TextureBase<N, TYP> {
    pub fn new<const BI: usize>(bn: &mut tex_binder::BOUNCER<BI>) -> tex_binder::Unbound<N, TYP> {
        let mut r = TextureBase { id: 0 };
        unsafe {
            gl::GenTextures(1, &mut r.id);
        }
        let mut r = tex_binder::Unbound::from(r);
        {
            // Need to set scale filter otherwise the texture can't be used so we set a reasonable default here to avoid errors
            let mut r = r.bind(bn);
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
            let mut r = r.bind(bn);
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
