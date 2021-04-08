use crate::{unwrap_result_or_ret, HasGLEnum};
use gl::types::*;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem::size_of;

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

impl<'a, ET> VBO<'a, ET> {
    pub fn new(elem_per_vert: &'a [u8]) -> Self {
        VBO::<ET>(BOBase::<ET>::new(), elem_per_vert)
    }

    pub fn with_data(elem_per_vert: &'a [u8], data: &[ET], usage: GLenum) -> Result<Self, String> {
        let mut r = Self::new(elem_per_vert);
        r.bind_bo();
        r.upload_to_bound_bo(data, usage)?;
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
    pub fn new() -> Self {
        IBO::<ET>(BOBase::<ET>::new())
    }

    pub fn with_data(data: &[ET], usage: GLenum) -> Result<Self, String> {
        let mut r = Self::new();
        r.bind_bo();
        r.upload_to_bound_bo(data, usage)?;
        Ok(r)
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
