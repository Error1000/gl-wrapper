use crate::render::program;
use crate::unwrap_result_or_ret;
use crate::util::buffer_obj;
use crate::HasGLEnum;
use gl::types::*;
use std::convert::TryFrom;
use std::mem::size_of;
use std::ptr;

impl vao_binder::OnBind for VAO{
    fn on_bind<const BI: usize>(&mut self) {
        self.bind_ao();
    }
}

pub type BoundVAO<'a> = vao_binder::Bound<'a, 0>;
pub type UnboundVAO = vao_binder::Unbound;
pub type VAOBouncer = vao_binder::BOUNCER<0>;

mod vao_binder {
    const NBOUNCERS: usize = 1;
    type Usable = super::VAO;

    pub trait OnBind {
        fn on_bind<const BI: usize>(&mut self);
    }

    use bitvec::prelude::*;
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
    pub struct Bound<'a, const BI: usize>(&'a mut Usable, &'a mut BOUNCER<BI>);

    impl<const BI: usize> Deref for Bound<'_, BI> {
        type Target = Usable;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<const BI: usize> DerefMut for Bound<'_, BI> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    pub struct Unbound(Usable)
    where
        Usable: OnBind; // Usable is private, this is important because it means to get a Usable you must go through bind which goes through a Bound which requires a &mut BOUNCER, whichs is unique, so no matter how many Unbound there are, there will only ever be one Bound at a time

    impl Unbound {
        #[inline]
        pub fn from(val: Usable) -> Unbound {
            Unbound(val)
        } // Takes a Usable and makes it an Unbound, this is fine since Usable can control how it's constructed and return an Unbound(Usable) instead of a Usable so there is no way a normal user can make a Usable without it being Unbound
        #[inline]
        pub fn bind<'a, const BI: usize>(&'a mut self, bn: &'a mut BOUNCER<BI>) -> Bound<'a, BI> {
            self.0.on_bind::<BI>();
            LAST_BOUND.store(BI, core::sync::atomic::Ordering::Relaxed);
            Bound(&mut self.0, bn)
        }
    }
}


pub struct VAO {
    id: GLuint,
    available_ind: Vec<GLuint>,
}

impl Drop for VAO {
    fn drop(self: &mut Self) {
        unsafe {
            gl::DeleteVertexArrays(1, &(self.id));
        }
    }
}

impl VAO {
    pub fn new() -> UnboundVAO {
        let mut r = VAO {
            id: 0,
            available_ind: Vec::new(),
        };
        unsafe {
            gl::GenVertexArrays(1, &mut (r.id));
        }
        UnboundVAO::from(r)
    }

    fn bind_ao(self: &Self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn adapt_vao_to_program(self: &mut Self, p: &program::Program) -> Result<(), ()> {
        for l in p.get_attribute_hashmap().values() {
            // If the data index the program needs has not been attached throw error so it is
            // impossible to cause undefined behaviour
            if !self.available_ind.contains(&l) {
                return Err(());
            }
            unsafe {
                gl::EnableVertexAttribArray(*l);
            }
        }
        Ok(())
    }

    /// Note: the auto_convert_to_f32 is here because some pretty old
    /// gpus can only work with f32,
    /// however this dosen't actually convert the VBO
    /// it just sets a flag that tells the gpu to convert to f32
    pub fn attach_vbo_to_vao<ET>(
        self: &mut Self,
        bo: &buffer_obj::VBO<ET>,
        index: GLuint,
        stride_ind: usize,
        auto_convert_to_f32: bool,
    ) -> Result<(), String>
    where
        ET: HasGLEnum,
    {
        // These are all i32 because that's what the opengl function takes (GLint)
        let mut sum: GLint = 0;
        for e in bo.get_elem_per_vertex() {
            sum += GLint::from(*e);
        }
        sum -= GLint::from(bo.get_elem_per_vertex()[stride_ind]);
        let skip: GLint = sum
            * unwrap_result_or_ret!(
                GLint::try_from(size_of::<ET>()),
                Err("Type size too big for opengl!".to_owned())
            );
        if skip < 0 {
            return Err("Size of elements computed was negative( this should have been impossible, maybe an integer overflow happened?)".to_owned());
        }

        let gl_typ = ET::get_gl_type();
        let is_int = gl_typ == GLbyte::get_gl_type()
            || gl_typ == GLubyte::get_gl_type()
            || gl_typ == GLshort::get_gl_type()
            || gl_typ == GLushort::get_gl_type()
            || gl_typ == GLint::get_gl_type()
            || gl_typ == GLuint::get_gl_type();

        if gl_typ == GLfloat::get_gl_type() || (auto_convert_to_f32 && is_int) {
            /*
                Docs say:
                For glVertexAttribPointer, if normalized is set to GL_TRUE, it indicates that values stored in an integer format are to be mapped to the range [-1,1] (for signed values) or [0,1] (for unsigned values) when they are accessed and converted to floating point. Otherwise, values will be converted to floats directly without normalization.
                so if we don't want to convert to float we should use glVertexAttribIPointer, even though glVertexAttrib support GL_BYTE, GL_UNSIGNED_BYTE, GL_SHORT, GL_UNSIGNED_SHORT, GL_INT and GL_UNSIGNED int too
            */
            unsafe {
                gl::VertexAttribPointer(
                        index,
                        bo.get_elem_per_vertex()[stride_ind].into(),
                        gl_typ,
                        gl::FALSE,
                        skip, // how many elements to skip each iteration
                        (ptr::null() as *const std::ffi::c_void).offset(unwrap_result_or_ret!(isize::try_from(skip), Err("Architecture size too small, stride offset too big, you put too many elements per vertex, so many that the cpu's architecutre can't properly hold the offset pointer so it can know where the elements for each attribute begin, i'm both impressed and horrified at the same time :)".to_owned()))), // offset by stride once ( not every iteration ) to make sure skipping works and that we are reading the right elements
                    );
            }
        } else if is_int {
            /*
                Docs say:
                For glVertexAttribIPointer, only the integer types GL_BYTE, GL_UNSIGNED_BYTE, GL_SHORT, GL_UNSIGNED_SHORT, GL_INT, GL_UNSIGNED_INT are accepted. Values are always left as integer values.
            */
            unsafe {
                gl::VertexAttribIPointer(
                    index,
                    bo.get_elem_per_vertex()[stride_ind].into(),
                    gl_typ,
                    skip, // how many elements to skip each iteration
                    (ptr::null() as *const std::ffi::c_void).offset(unwrap_result_or_ret!(isize::try_from(skip), Err("Architecture size too small, stride offset too big, you put too many elements per vertex, so many that the cpu's architecutre can't properly hold the offset pointer so it can know where the elements for each attribute begin, i'm both impressed and horrified at the same time :)".to_owned()))), // offset by stride once ( not every iteration ) to make sure skipping works and that we are reading the right elements
                );
            }
        } else if gl_typ == GLdouble::get_gl_type() {
            /*
                Docs say:
                GL_DOUBLE is also accepted by glVertexAttribLPointer and is the only token accepted by the type parameter for that function.
            */
            unsafe {
                gl::VertexAttribLPointer(
                    index,
                    bo.get_elem_per_vertex()[stride_ind].into(),
                    gl_typ,
                    skip, // how many elements to skip each iteration
                    (ptr::null() as *const std::ffi::c_void).offset(unwrap_result_or_ret!(isize::try_from(skip), Err("Architecture size too small, stride offset too big, you put too many elements per vertex, so many that the cpu's architecutre can't properly hold the offset pointer so it can know where the elements for each attribute begin, i'm both impressed and horrified at the same time :)".to_owned()))), // offset by stride once ( not every iteration ) to make sure skipping works and that we are reading the right elements
                );
            }
        } else {
            return Err(String::from("Invalid data type for opengl!"));
        }

        self.available_ind.push(index);
        Ok(())
    }
}
