extern crate gl_wrapper;
extern crate glutin;
extern crate image;

use gl_wrapper::render::texture::TextureFunc;
use gl_wrapper::render::*;
use gl_wrapper::util::buffer_obj::BOFunc;
use gl_wrapper::util::*;

use glutin::dpi::PhysicalSize;
use image::GenericImageView;
use std::convert::TryInto;
use std::ffi::CStr;

use gl::types::*;
use std::ptr;
use std::str;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;

use std::path::Path;

// Vertex data
static VERTEX_DATA: [GLfloat; 8] = [-1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0];

// Tex data
static TEX_DATA: [GLfloat; 8] = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];

// Indices data
static IND_DATA: [GLushort; 6] = [0, 1, 3, 1, 2, 3];

// Shader sources
static VS_SRC: &str = "
#version 150
attribute vec2 position;
attribute vec2 tex_coord;
out vec2 pass_tex_coord;
void main() {
    pass_tex_coord = tex_coord;
    gl_Position = vec4(position, 0.0, 1.0);
}";

static FS_SRC: &str = "
#version 150
out vec4 out_color;
uniform sampler2D obj_tex;
in vec2 pass_tex_coord;
void main() {
    out_color = texture2D(obj_tex, pass_tex_coord);
}";

fn main() {
    let events_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: 400.0,
            height: 400.0,
        })
        .with_visible(false); // Hide window while loading to make it less annoying
    let gl_window = glutin::ContextBuilder::new()
        .build_windowed(window, &events_loop)
        .expect("Failed to create window!");
    let gl_window = unsafe { gl_window.make_current() }.unwrap();

    // Load the OpenGL function pointers
    gl_wrapper::api::init(&gl_window);
    println!(
        "Window created but hidden so that we don't have an annoying hang as the program loads!"
    );

    let v = unsafe {
        let data = CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };

    println!("OpenGL Version: {}", v);

    // Create GLSL shaders
    println!("Loading shaders ...");
    let mut program = {
        // Program and shader provide their own error messages
        let vs = shader::VertexShader::new(VS_SRC).unwrap();
        let fs = shader::FragmentShader::new(FS_SRC).unwrap();
        program::Program::new(&[&vs.into(), &fs.into()]).unwrap()
    };

    program.bind_program();
    program
        .load_attribute("position")
        .expect("Failed to load attribute from shader!");
    program
        .load_attribute("tex_coord")
        .expect("Failed to load attribute from shader!");
    program
        .load_sampler("obj_tex")
        .expect("Failed to load attribute from shader!");
    println!("Done!");

    // Load textures
    println!("Loading textures ...");
    let im = image::open(&Path::new("apple.png"))
        .expect("Failed to read texture from disk! Are you sure it exists?");
    let data: &[u8] = &im.to_bytes()[..];
    let mut t = texture::Texture2D::new();
    t.bind_texture_for_data();
    t.upload_data_to_bound_texture([
                                       im.width().try_into().expect("Image size is either too big or negative, in any of those cases that's not a good sign!"),
                                       im.height().try_into().expect("Image size is either too big or negative, in any of those cases that's not a good sign!")
                                        ],
                                    data,
                                   4 /* RGBA has 4 channel per pixel*/
    ).expect("Failed to upload texture data to gpu!");
    t.bind_texture_for_sampling(program.get_sampler_id("obj_tex"));
    println!("Done!");

    // Load mesh data ( indices, vertices, uv data )
    println!("Loading mesh ...");
    let mut a = aggregator_obj::VAO::new();
    a.bind_vao_for_data();

    // NOTE: Creating a vbo with data auto binds it, creating a vbo using new does not
    let pos_vbo = buffer_obj::VBO::<GLfloat>::with_data(2, &VERTEX_DATA, gl::STATIC_DRAW)
        .expect("Failed to upload data to vbo!");
    a.attach_bound_vbo_to_bound_vao(&pos_vbo, program.get_attribute_id("position"))
        .expect("Failed to attach vob to vao!");

    let tex_vbo = buffer_obj::VBO::<GLfloat>::with_data(2, &TEX_DATA, gl::STATIC_DRAW)
        .expect("Failed to upload data to vbo!");
    a.attach_bound_vbo_to_bound_vao(&tex_vbo, program.get_attribute_id("tex_coord"))
        .expect("Failed to attach vbo to vao!");

    a.bind_vao_for_program(&program).expect("Shader is asking for more values than vao has attached, all attributes the shader uses must be attached to vao!");

    let mut ind_ibo = buffer_obj::IBO::<GLushort>::new();
    ind_ibo.bind_bo();
    ind_ibo
        .upload_to_bound_bo(&IND_DATA, gl::STATIC_DRAW)
        .expect("Failed to upload data to ibo!");
    println!("Done!");

    println!("Showing window!");
    gl_window.window().set_visible(true);

    unsafe {
        gl::ClearColor(0.0, 0.0, 1.0, 1.0);
    }
    // Since these values won't change and the gl::DrawElements is in the hot path we are going to cache these values now just to make things simpler and faster
    let ibo_len = ind_ibo
        .get_size()
        .try_into()
        .expect("The number of triangles you have is either negative, or too big!");
    let ibo_enum_type = gl_wrapper::api::type_to_gl_enum::<GLushort>().unwrap();
    let render = move || unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::DrawElements(gl::TRIANGLES, ibo_len, ibo_enum_type, ptr::null());
    };

    events_loop.run(move |event, _, control_flow| {
        // Unless we re write the control flow just wait until another evetn arrives when this iteration finished
        *control_flow = ControlFlow::Wait;
        match event {
            // Window stuff
            Event::WindowEvent { event, .. } => {
                match event{
                    WindowEvent::Resized(PhysicalSize{width, height}) => {
                        unsafe {
                            gl::Viewport(0, 0, width.try_into().expect("Either your window's width has a negative size(wut?) or is too big(how even?), opengl can't handle that number!"),
                                         height.try_into().expect("Either your window' height has a negative size(wut?) or is too big(how even?), opengl can't handle that number!"));
                        }
                        render();
                    },
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input: glutin::event::KeyboardInput{ virtual_keycode: Some(glutin::event::VirtualKeyCode::Escape), .. },
                     .. } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            },
            // Rendering stuff
            Event::RedrawRequested(..) => {
                render();
                gl_window.swap_buffers().unwrap();
            },
            _ => {}
        }
    });
}
