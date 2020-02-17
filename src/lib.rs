use gl::types::*;
use glutin::PossiblyCurrent;
use glutin::{NotCurrent, WindowedContext};
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

pub trait HasGLEnum{
    fn get_gl_enum() -> GLenum;
}

impl HasGLEnum for GLfloat{
    #[inline(always)]
    fn get_gl_enum() -> GLenum { gl::FLOAT }
}

impl HasGLEnum for GLint{
    #[inline(always)]
    fn get_gl_enum() -> GLenum { gl::INT }
}
impl HasGLEnum for GLuint{
    #[inline(always)]
    fn get_gl_enum() -> GLenum{ gl::UNSIGNED_INT }
}

impl HasGLEnum for GLshort{
    #[inline(always)]
    fn get_gl_enum() -> GLenum { gl::SHORT }
}

impl HasGLEnum for GLushort{
    #[inline(always)]
    fn get_gl_enum() -> GLenum { gl::UNSIGNED_SHORT }
}

impl HasGLEnum for GLubyte{
    #[inline(always)]
    fn get_gl_enum() -> GLenum { gl::UNSIGNED_BYTE }
}

impl HasGLEnum for GLbyte{
    #[inline(always)]
    fn get_gl_enum() -> GLenum { gl::BYTE }
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

pub fn init(win: WindowedContext<NotCurrent>) -> Option<WindowedContext<PossiblyCurrent>> {
    let w = unwrap_or_ret_none!(unsafe { win.make_current() });
    gl::load_with(|symbol| w.get_proc_address(symbol));
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
        Some(w)
    }
}

#[inline]
pub fn format_to_gl_internal_format(bpc: u8, format: GLenum) -> Option<(GLint, u8)> {
    let cpp: u8 = match format {
        gl::RED => 1,
        gl::RG => 2,
        gl::RGB => 3,
        gl::RGBA => 4,
        _ => return None,
    };

    let internal_format = match bpc {
        8 => match format {
            gl::RED => gl::R8,
            gl::RG => gl::RG8,
            gl::RGB => gl::RGB8,
            gl::RGBA => gl::RGBA8,
            _ => return None,
        },

        16 => match format {
            gl::RED => gl::R16,
            gl::RG => gl::RG16,
            gl::RGB => gl::RGB16,
            gl::RGBA => gl::RGBA16,
            _ => return None,
        },

        _ => return None,
    };
    Some((
        internal_format
            .try_into()
            .expect("FATAL Failure, faulty opengl implementation!"),
        cpp,
    ))
}
