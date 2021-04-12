use crate::{unwrap_result_or_ret, HasGLEnum};
use gl::types::*;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem::size_of;

/**********************************************************/
impl<'b, ET> vbo_binder::OnBind for VBO<'b, ET> {
    #[inline(always)]
    fn on_bind<const BI: usize>(&mut self) {
        self.bind_bo();
    }
}

pub type VBOBouncer = vbo_binder::BOUNCER<0>;
pub type BoundVBO<'a, 'b, ET> = vbo_binder::Bound<'a, 'b, ET, 0>;
pub type UnboundVBO<'b, ET>   = vbo_binder::Unbound<'b, ET>;

mod vbo_binder{
    const NBOUNCERS: usize = 1;
    type Usable<'b, ET> = super::VBO<'b, ET>;
    
    pub trait OnBind{
        fn on_bind<const BI: usize>(&mut self);
    }
    
    use std::{ops::{Deref, DerefMut}, sync::{Mutex, atomic::AtomicUsize}};
    use bitvec::prelude::*;

    lazy_static!{
        static ref BOUNCER_GUARD: Mutex<BitArr!(for NBOUNCERS, in Msb0, u8)> = Mutex::new(BitArray::zeroed()); // BOUNCER_GUARD is private, this is important because we don't want somebody take()-ing the intialised OnceCell, leaving it uninitialised, and being able to call new() again on BOUNCER again and have two BOUNCERs
        /// SAFTEY: LAST_BOUND is unreliable, don't rely on it for correctness
        pub static ref LAST_BOUND: AtomicUsize = AtomicUsize::new(0);
    }

    pub struct BOUNCER<const BI: usize>(()); // NOTE: () is private, this is important so that the only way to get a BOUNCER instance is to use new()
    
    impl<const BI: usize> BOUNCER<BI>{
        /// IMPORTANT: Only one bouncer can exist ever
        /// SAFETY: We are the only ones who can access BOUNCER_GUARD because it is private and we can use that to make sure that we only create one BOUNCER ever
        #[inline]
        pub fn new() -> Option<Self>{
            if BI >= NBOUNCERS{ return None; }
            let mut lck = BOUNCER_GUARD.try_lock().ok()?;
            if lck.get(BI).unwrap() == false{
                lck.set(BI, true);
                Some(BOUNCER(()))
            }else{ None }
    
        }
    }
    
    // Because there only ever exists one bouncer a &mut to a BOUNCER must be unique, so thre can only ever exist one Bound
    pub struct Bound<'a, 'b, ET, const BI: usize>(&'a mut Usable<'b, ET>, &'a mut BOUNCER<BI>);
    
    impl<'b, ET, const BI: usize> Deref for Bound<'_, 'b, ET, BI>{
        type Target = Usable<'b, ET>;
    
        #[inline]
        fn deref(&self) -> &Self::Target { &self.0 }
    }
    
    impl<'b, ET, const BI: usize> DerefMut for Bound<'_, 'b, ET, BI>{
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
    }
    
    pub struct Unbound<'b, ET>(Usable<'b, ET>) where Usable<'b, ET>: OnBind; // Usable is private, this is important because it means to get a Usable you must go through bind which goes through a Bound which requires a &mut BOUNCER, whichs is unique, so no matter how many Unbound there are, there will only ever be one Bound at a time
    
    impl<'b, ET> Unbound<'b, ET> {
        #[inline]
        pub fn from(val: Usable<'b, ET>) -> Unbound<'b, ET> { Unbound(val) } // Takes a Usable and makes it an Unbound, this is fine since Usable can control how it's constructed and return an Unbound(Usable) instead of a Usable so there is no way a normal user can make a Usable without it being Unbound
        #[inline]
        pub fn bind<'a, const BI: usize>(&'a mut self, bn: &'a mut BOUNCER<BI>) -> Bound<'a, 'b, ET, BI> {
            self.0.on_bind::<BI>();
            LAST_BOUND.store(BI, core::sync::atomic::Ordering::Relaxed);
            Bound(&mut self.0, bn)
        }
    }
    
}


/**********************************************************/

impl<ET> ibo_binder::OnBind for IBO<ET>{
    fn on_bind<const BI: usize>(&mut self) {
        self.bind_bo();
    }
}

pub type IBOBouncer = ibo_binder::BOUNCER<0>;
pub type UnboundIBO<ET> = ibo_binder::Unbound<ET>;
pub type BoundIBO<'a, ET> = ibo_binder::Bound<'a, ET, 0>;

mod ibo_binder {
    const NBOUNCERS: usize = 1;
    type Usable<ET> = super::IBO<ET>;

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
    pub struct Bound<'a, ET, const BI: usize>(&'a mut Usable<ET>, &'a mut BOUNCER<BI>);

    impl<ET, const BI: usize> Deref for Bound<'_, ET, BI> {
        type Target = Usable<ET>;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<ET, const BI: usize> DerefMut for Bound<'_, ET, BI> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    pub struct Unbound<ET>(Usable<ET>)
    where
        Usable<ET>: OnBind; // Usable is private, this is important because it means to get a Usable you must go through bind which goes through a Bound which requires a &mut BOUNCER, whichs is unique, so no matter how many Unbound there are, there will only ever be one Bound at a time

    impl<ET> Unbound<ET> {
        #[inline]
        pub fn from(val: Usable<ET>) -> Unbound<ET> {
            Unbound(val)
        } // Takes a Usable and makes it an Unbound, this is fine since Usable can control how it's constructed and return an Unbound(Usable) instead of a Usable so there is no way a normal user can make a Usable without it being Unbound
        #[inline]
        pub fn bind<'a, /*TYPE_GENERICS_HERE*/ const BI: usize>(&'a mut self, bn: &'a mut BOUNCER<BI>) -> Bound<'a, ET, BI> {
            self.0.on_bind::<BI>();
            LAST_BOUND.store(BI, core::sync::atomic::Ordering::Relaxed);
            Bound(&mut self.0, bn)
        }
    }
}



/**********************************************************/

pub struct BOBase<ET> {
    id: GLuint,
    size: GLsizeiptr, // Opengl uses signed integers for size
    /// Used here to link each BO with it's data type on gpu to make sure no problems can happen from uploading data of the wrong datatype and corrupting buffers
    data: PhantomData<ET>,
}

impl<ET> BOBase<ET> {
    fn new() -> Self {
        let mut r = BOBase::<ET> {
            id: 0,
            size: 0,
            data: PhantomData,
        };
        unsafe {
            gl::GenBuffers(1, &mut r.id);
        }
        r
    }
}

impl<ET> Drop for BOBase<ET> {
    fn drop(self: &mut Self) {
        // Drop ET array on gpu
        unsafe {
            gl::DeleteBuffers(1, &(self.id));
        }
    }
}

/// Use an array for elem_per_vert to allow striding
pub struct VBO<'a, ET>(BOBase<ET>, &'a [u8]);
pub struct IBO<ET>(BOBase<ET>);

impl<'a, ET: 'a> VBO<'a, ET> {
    pub fn new(elem_per_vert: &'a [u8]) -> vbo_binder::Unbound<'a, ET> {
        vbo_binder::Unbound::<'a, ET>::from(VBO::<ET>(BOBase::<ET>::new(), elem_per_vert))
    }

    pub fn with_data(
        bn: &mut VBOBouncer,
        elem_per_vert: &'a [u8],
        data: &[ET],
        usage: GLenum,
    ) -> Result<vbo_binder::Unbound<'a, ET>, String> {
        let mut r = Self::new(elem_per_vert);
        {
            let mut r = r.bind(bn);
            r.upload_to_bound_bo(data, usage)?;
        }
        Ok(r)
    }

    pub fn get_elem_per_vertex(self: &Self) -> &'a [u8] {
        self.1
    }

    // NOTE: This dosen't just return a value it uses two values in the struct to compute this value so it's not as lightweight as other getters
    pub fn get_num_of_vertices(self: &Self) -> GLsizeiptr {
        let mut sum: GLsizeiptr = 0;
        for e in self.get_elem_per_vertex() {
            sum += GLsizeiptr::from(*e);
        }
        self.get_size() / sum
    }

    pub fn upload_to_bound_bo(self: &mut Self, data: &[ET], usage: GLenum) -> Result<(), String> {
        self.0.size = unwrap_result_or_ret!(
            GLsizeiptr::try_from(data.len()),
            Err("Too many elements in data slice for opengl!".to_owned())
        );
        unsafe {
            gl::BufferData(
                Self::get_gl_type(),
                self.get_size()
                    * unwrap_result_or_ret!(
                        GLsizeiptr::try_from(size_of::<ET>()),
                        Err("Invalid size of data type, how even?".to_owned())
                    ),
                &data[0] as *const ET as *const std::ffi::c_void,
                usage,
            );
        }
        Ok(())
    }
}

impl<ET> IBO<ET> {
    pub fn new() -> UnboundIBO<ET> {
        UnboundIBO::from(IBO::<ET>(BOBase::<ET>::new()))
    }

    pub fn with_data(bn: &mut IBOBouncer, data: &[ET], usage: GLenum) -> Result<UnboundIBO<ET>, String> {
        let mut r = Self::new();
        {
            let mut r = r.bind(bn);
            r.upload_to_bo(data, usage)?;
        }
        Ok(r)
    }

    pub fn upload_to_bo(self: &mut Self, data: &[ET], usage: GLenum) -> Result<(), String> {
        self.0.size = unwrap_result_or_ret!(
            GLsizeiptr::try_from(data.len()),
            Err("Too many elements in data slice for opengl!".to_owned())
        );
        unsafe {
            gl::BufferData(
                Self::get_gl_type(),
                self.get_size()
                    * unwrap_result_or_ret!(
                        GLsizeiptr::try_from(size_of::<ET>()),
                        Err("Invalid size of data type, how even?".to_owned())
                    ),
                &data[0] as *const ET as *const std::ffi::c_void,
                usage,
            );
        }
        Ok(())
    }
}

pub trait BOFunc<ET>
where
    Self: HasGLEnum,
{
    #[inline(always)]
    fn bind_bo(self: &Self) {
        unsafe { gl::BindBuffer(Self::get_gl_type(), self.get_bo_base().id) }
    }

    #[inline(always)]
    fn get_size(self: &Self) -> GLsizeiptr {
        self.get_bo_base().size
    }
    fn get_bo_base(self: &Self) -> &BOBase<ET>;
}

unsafe impl<'a, ET> HasGLEnum for VBO<'a, ET> {
    #[inline(always)]
    fn get_gl_type() -> GLenum {
        gl::ARRAY_BUFFER
    }
}

unsafe impl<ET> HasGLEnum for IBO<ET> {
    #[inline(always)]
    fn get_gl_type() -> GLenum {
        gl::ELEMENT_ARRAY_BUFFER
    }
}

impl<'a, ET> BOFunc<ET> for VBO<'a, ET> {
    #[inline(always)]
    fn get_bo_base(self: &Self) -> &BOBase<ET> {
        &self.0
    }
}

impl<ET> BOFunc<ET> for IBO<ET> {
    #[inline(always)]
    fn get_bo_base(self: &Self) -> &BOBase<ET> {
        &self.0
    }
}
