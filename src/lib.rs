use gl::types::*;
use glutin::window::Window;
use glutin::ContextWrapper;
use glutin::PossiblyCurrent;
use std::any::TypeId;
use std::convert::TryInto;
use std::ffi::CStr;

pub mod render;
pub mod util;

#[macro_export]
macro_rules! unwrap_or_ret_none {
    ($x:expr) => {
        match $x {
            Ok(val) => val,
            Err(_) => return None,
        }
    };
}

// NOTE about design: To future self never have getters that get a mutable self reference or return a mutable reference to a value no matter what
#[inline]
pub fn get_gl_version_str() -> String {
    unsafe {
        CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8)
            .to_string_lossy()
            .into_owned()
    }
}

// NOTE: We use always here to make sure the optimiser gets the best chance to remove ethe bounds checks
#[inline(always)]
pub fn set_gl_clear_color(r: f32, g: f32, b: f32, a: f32) {
    if r > 1.0 || r < 0.0 {
        panic!("R value of clear color too big or too small ( has to be between 0.0 and 1.0 )!");
    } else if g > 1.0 || g < 0.0 {
        panic!("G value of clear color too big or too small ( has to be between 0.0 and 1.0 )!");
    } else if b > 1.0 || b < 0.0 {
        panic!("B value of clear color too big or too small ( has to be between 0.0 and 1.0 )!");
    }
    unsafe {
        gl::ClearColor(r, g, b, a);
    }
}

#[inline]
pub fn set_gl_draw_size(w: u32, h: u32) -> Option<()> {
    unsafe {
        gl::Viewport(
            0,
            0,
            unwrap_or_ret_none!(w.try_into()),
            unwrap_or_ret_none!(h.try_into()),
        );
    }
    Some(())
}

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

#[inline]
pub fn shader_glenum_to_string(e: GLenum) -> Option<&'static str> {
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

#[inline]
pub fn texture_bits_to_gl_types(bpc: u8, cpp: u8) -> Option<(GLint, GLenum)> {
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
    Some((
        internal_format
            .try_into()
            .expect("FATAL Failure, faulty opengl implementation!"),
        format,
    ))
}
