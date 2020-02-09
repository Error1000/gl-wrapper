use gl::types::*;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem::size_of;

pub struct BaseBO<ET>{
    id: GLuint,
    size: isize,
    data: PhantomData<ET>,
}

impl<ET> BaseBO<ET>{
    fn new() -> Self{
        let mut r = BaseBO::<ET>{
            id: 0,
            size: 0,
            data: PhantomData,
        };
        unsafe { gl::GenBuffers(1, &mut r.id); }
        r
    }
}

impl<ET> Drop for BaseBO<ET>{
    fn drop(self: &mut Self) {
        unsafe { gl::DeleteBuffers(1, &(self.id)); }
    }
}



pub struct VBO<ET> (BaseBO<ET>, u8);
pub struct IBO<ET> (BaseBO<ET>);


impl<ET> VBO<ET> {
    pub fn new(elem_per_vert: u8) -> Self {
        VBO::<ET> (BaseBO::<ET>::new(), elem_per_vert)
    }

    pub fn with_data(elem_per_vert: u8, data: &[ET], usage: GLenum) -> Option<Self>{
        let mut r = Self::new(elem_per_vert);
        r.bind_bo();
        r.upload_to_bound_bo(data, usage)?;
        Some(r)
    }

    pub fn get_elem_per_vertex(self: &Self) -> u8 { self.1 }

    // NOTE: This dosen't just return a value it uses two values already in the struct to get this value so it's not as lightwieght as other getters
    pub fn get_num_of_vertices(self: &Self) -> isize {
        self.get_size() / (self.get_elem_per_vertex() as isize)
    }

    // TODO: Consider renaming this
    pub fn upload_to_bound_bo(self: &mut Self, data: &[ET], usage: GLenum) -> Option<()>{
        if size_of::<ET>() > std::isize::MAX as usize { return None; }
        if data.len() > std::isize::MAX as usize { return None; }
        self.0.size = match data.len().try_into() {
            Ok(val) => val,
            Err(_) => return None,
        };
        unsafe {
            gl::BufferData(
                Self::get_type(),
                self.get_size() * (size_of::<ET>() as isize),
                &data[0] as *const ET as *const std::ffi::c_void,
                usage,
            );
        }
        Some(())
    }
}

impl<ET> IBO<ET> {
   pub fn new() -> Self {
       IBO::<ET> ( BaseBO::<ET>::new() )
   }

    // TODO: Consider renaming this
    pub fn upload_to_bound_bo(self: &mut Self, data: &[ET], usage: GLenum) -> Option<()>{
        if size_of::<ET>() > std::isize::MAX as usize { return None; }
        if data.len() > std::isize::MAX as usize { return None; }
        self.0.size = match data.len().try_into() {
            Ok(val) => val,
            Err(_) => return None,
        };
        unsafe {
            gl::BufferData(
                Self::get_type(),
                self.get_size() * (size_of::<ET>() as isize),
                &data[0] as *const ET as *const std::ffi::c_void,
                usage,
            );
        }
        Some(())
    }
}



pub trait BOFunc<ET> {
    fn bind_bo(self: &Self){
        unsafe { gl::BindBuffer(Self::get_type(), self.get_base_bo().id) }
    }

    fn get_size(self: &Self) -> isize;
    fn get_base_bo(self: &Self) -> &BaseBO<ET>;
    fn get_type() -> GLenum;
}


impl<ET> BOFunc<ET> for VBO<ET> {
    #[inline]
    fn get_size(self: &Self) -> isize { self.get_base_bo().size }
    #[inline]
    fn get_base_bo(self: &Self) -> &BaseBO<ET> { &self.0 }
    #[inline]
    fn get_type() -> GLenum { gl::ARRAY_BUFFER }
}

impl<ET> BOFunc<ET> for IBO<ET> {
    #[inline]
    fn get_size(self: &Self) -> isize { self.get_base_bo().size }
    #[inline]
    fn get_base_bo(self: &Self) -> &BaseBO<ET> { &self.0 }
    #[inline]
    fn get_type() -> GLenum { gl::ELEMENT_ARRAY_BUFFER }
}
