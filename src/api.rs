use gl::types::*;
use glutin::window::Window;
use glutin::ContextWrapper;
use glutin::PossiblyCurrent;
use std::any::TypeId;
use std::convert::TryInto;

pub fn type_to_gl_enum<T: 'static>() -> Option<GLenum> {
    if TypeId::of::<T>() == TypeId::of::<GLfloat>() {
        Some(gl::FLOAT)
    } else if TypeId::of::<T>() == TypeId::of::<GLint>() {
        Some(gl::INT)
    } else if TypeId::of::<T>() == TypeId::of::<GLuint>() {
        Some(gl::UNSIGNED_INT)
    } else if TypeId::of::<T>() == TypeId::of::<GLshort>() {
        Some(gl::SHORT)
    } else if TypeId::of::<T>() == TypeId::of::<GLushort>() {
        Some(gl::UNSIGNED_SHORT)
    } else if TypeId::of::<T>() == TypeId::of::<GLubyte>() {
        Some(gl::UNSIGNED_BYTE)
    } else if TypeId::of::<T>() == TypeId::of::<GLbyte>() {
        Some(gl::BYTE)
    } else {
        None
    }
}

pub fn gl_shader_enum_to_string(e: GLenum) -> Option<&'static str> {
    match e {
        gl::VERTEX_SHADER => Some("vertex shader"),
        gl::FRAGMENT_SHADER => Some("fragment shader"),
        gl::GEOMETRY_SHADER => Some("geometry shader"),
        _ => None,
    }
}

pub fn init(w: &ContextWrapper<PossiblyCurrent, Window>) {
    gl::load_with(|symbol| w.get_proc_address(symbol));
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
    }
}

pub fn texture_bits_to_opengl_types(bpc: u8, cpp: u8) -> Option<(GLint, GLenum)> {
    let format = match cpp {
        1 => gl::RED,
        2 => gl::RG,
        3 => gl::RGB,
        4 => gl::RGBA,
        _ => return None,
    };
    let internal_format = match bpc {
        8 => match cpp {
            1 => gl::R8,
            2 => gl::RG8,
            3 => gl::RGB8,
            4 => gl::RGBA8,
            _ => return None,
        },

        16 => match cpp {
            1 => gl::R16,
            2 => gl::RG16,
            3 => gl::RGB16,
            4 => gl::RGBA16,
            _ => return None,
        },

        _ => return None,
    };
    return Some((
        internal_format
            .try_into()
            .expect("FATAL Failure, faulty opengl implementation!"),
        format,
    ));
}
