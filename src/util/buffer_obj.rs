use crate::unwrap_result_or_ret;
use gl::types::*;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem::size_of;

pub struct BOBase<ET> {
    id: GLuint,
    size: isize,
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
    pub fn get_size(self: &Self) -> isize {
        self.size
    }
}

impl<ET> Drop for BOBase<ET> {
    fn drop(self: &mut Self) {
        unsafe {
            gl::DeleteBuffers(1, &(self.id));
        }
    }
}

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

    // NOTE: This dosen't just return a value it uses two values already in the struct to get this value so it's not as lightweight as other getters
    pub fn get_num_of_vertices(self: &Self) -> isize {
        let mut sum: isize = 0;
        for e in self.get_elem_per_vertex() {
            sum += isize::from(*e);
        }
        self.get_size() / sum
    }

    pub fn upload_to_bound_bo(self: &mut Self, data: &[ET], usage: GLenum) -> Result<(), String> {
        self.0.size = unwrap_result_or_ret!(
            isize::try_from(data.len()),
            Err("Size of data too big for opengl!".to_owned())
        );
        unsafe {
            gl::BufferData(
                Self::get_type(),
                self.get_size()
                    * unwrap_result_or_ret!(
                        isize::try_from(size_of::<ET>()),
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
            isize::try_from(data.len()),
            Err("Size of data is too big for opengl!".to_owned())
        );
        unsafe {
            gl::BufferData(
                Self::get_type(),
                self.get_size()
                    * unwrap_result_or_ret!(
                        isize::try_from(size_of::<ET>()),
                        Err("Invalid size of data type, how even?".to_owned())
                    ),
                &data[0] as *const ET as *const std::ffi::c_void,
                usage,
            );
        }
        Ok(())
    }
}

pub trait BOFunc<ET> {
    fn bind_bo(self: &Self) {
        unsafe { gl::BindBuffer(Self::get_type(), self.get_bo_base().id) }
    }

    fn get_size(self: &Self) -> isize;
    fn get_bo_base(self: &Self) -> &BOBase<ET>;
    fn get_type() -> GLenum;
}

impl<'a, ET> BOFunc<ET> for VBO<'a, ET> {
    #[inline]
    fn get_size(self: &Self) -> isize {
        self.get_bo_base().size
    }
    #[inline]
    fn get_bo_base(self: &Self) -> &BOBase<ET> {
        &self.0
    }
    #[inline]
    fn get_type() -> GLenum {
        gl::ARRAY_BUFFER
    }
}

impl<ET> BOFunc<ET> for IBO<ET> {
    #[inline]
    fn get_size(self: &Self) -> isize {
        self.get_bo_base().size
    }
    #[inline]
    fn get_bo_base(self: &Self) -> &BOBase<ET> {
        &self.0
    }
    #[inline]
    fn get_type() -> GLenum {
        gl::ELEMENT_ARRAY_BUFFER
    }
}
