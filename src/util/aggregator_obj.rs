use gl::types::*;
use std::ptr;
use crate::render::program;
use crate::unwrap_result_or_ret;
use crate::util::buffer_obj;
use crate::HasGLEnum;
use std::convert::TryFrom;
use std::mem::size_of;

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
    pub fn new() -> Self {
        let mut r = VAO {
            id: 0,
            available_ind: Vec::new(),
        };
        unsafe {
            gl::GenVertexArrays(1, &mut (r.id));
        }
        r
    }

    pub fn bind_ao(self: &Self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn adapt_bound_vao_to_program(self: &mut Self, p: &program::Program) -> Result<(), ()> {
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

    pub fn attach_bound_vbo_to_bound_vao<ET>(
        self: &mut Self,
        bo: &buffer_obj::VBO<ET>,
        index: GLuint,
        stride_ind: usize,
    ) -> Result<(), String>
    where
        ET: HasGLEnum,
    {
	// These are all i32 because that's what the opengl function takes (GLint)
        let mut sum: i32 = 0;
	for e in bo.get_elem_per_vertex(){
		sum += i32::from(*e);
	}
	sum -= i32::from(bo.get_elem_per_vertex()[stride_ind]);
	let skip: i32 = sum * unwrap_result_or_ret!(
                    i32::try_from(size_of::<ET>()),
                    Err("Type size too big for opengl!".to_owned()));
	if skip < 0 { return Err("Size of elemnts computed was negative( this should have been impossible, maybe an integer overflow happened?)".to_owned()); }
        unsafe {
            gl::VertexAttribPointer(
                index,
                bo.get_elem_per_vertex()[stride_ind].into(),
                ET::get_gl_enum(),
                gl::FALSE,
                skip, // how many elements to skip each iteration
                (ptr::null() as *const std::ffi::c_void).offset(unwrap_result_or_ret!(isize::try_from(skip), Err("Architecture size too small, stride offset too big, you put too many elements per vertex, so many that the cpu's architecutre can't properly hold the offset pointer so it can know where the elements for each attribute begin, i'm both impressed and horrified at the same time :)".to_owned()))), // offset by stride once ( not every iteration ) to make sure skipping works and that we are reading the right elements
            );
        }
        self.available_ind.push(index);
        Ok(())
    }
}
