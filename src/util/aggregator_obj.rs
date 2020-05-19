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

    pub fn bind_ao(self: &mut Self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn adapt_bound_vao_to_program(self: &mut Self, p: &program::Program) -> Option<()> {
        for l in p.get_attribute_hashmap().values() {
            // If the data index the program needs has not been attached throw error so it is
            // impossible to cause undefined behaviour
            if !self.available_ind.contains(&l) {
                return None;
            }
            unsafe {
                gl::EnableVertexAttribArray(*l);
            }
        }
        Some(())
    }

    //TODO: Rework
    pub fn attach_bound_vbo_to_bound_vao<ET>(
        self: &mut Self,
        bo: &buffer_obj::VBO<ET>,
        index: GLuint,
        stride: usize,
    ) -> Result<(), String>
    where
        ET: HasGLEnum,
    {
        let mut sum: i32 = 0;
        for i in 0..stride {
            sum += i32::from(bo.get_elem_per_vertex()[i]);
        }

        unsafe {
            gl::VertexAttribPointer(
                index,
                bo.get_elem_per_vertex()[stride].into(),
                ET::get_gl_enum(),
                gl::FALSE,
                sum * unwrap_result_or_ret!(
                    i32::try_from(size_of::<ET>()),
                    Err("Type too big for opengl!".to_owned())
                ),
                ptr::null(),
            );
        }
        self.available_ind.push(index);
        Ok(())
    }
}
