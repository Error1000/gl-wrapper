use gl::types::*;
use std::ptr;

use crate::api;
use crate::render::program;
use crate::util::buffer_obj;

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

    pub fn bind_vao_for_data(self: &mut Self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    /// TODO: Optimise this maybe
    pub fn bind_vao_for_program(self: &mut Self, p: &program::Program) -> Option<()> {
        self.bind_vao_for_data();
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

    pub fn attach_bound_vbo_to_bound_vao<ET: 'static>(
        self: &mut Self,
        bo: &buffer_obj::VBO<ET>,
        index: GLuint,
    ) -> Result<(), &'static str> {
        unsafe {
            let t = match api::type_to_gl_enum::<ET>() {
                Some(r) => r,
                None => {
                    return Err("Invalid type for buffer data (a.k.a the type of the elements of the buffer you just passed me is not supported)");
                }
            };
            gl::VertexAttribPointer(
                index,
                bo.get_elem_per_vertex().into(),
                t,
                gl::FALSE,
                0,
                ptr::null(),
            );
        }
        self.available_ind.push(index);
        Ok(())
    }
}
