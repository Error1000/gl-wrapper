extern crate gl_wrapper;
extern crate glutin;
extern crate image;

use gl_wrapper::render::texture::TextureFunc;
use gl_wrapper::render::*;
use gl_wrapper::util::buffer_obj::BOFunc;
use gl_wrapper::util::*;
use gl_wrapper::HasGLEnum;

use glutin::dpi::PhysicalSize;
use std::convert::TryInto;

use gl::types::*;
use std::ptr;
use std::str;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;

use glutin::platform::desktop::EventLoopExtDesktop;
use std::path::Path;
use std::time::Instant;

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
    let mut events_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: 400.0,
            height: 400.0,
        })
        .with_visible(false); // Hide window while loading to make it less annoying
    let gl_window = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window, &events_loop)
        .expect("Failed to create window!");

    // Load the OpenGL function pointers
    let gl_window = gl_wrapper::init(gl_window).expect("Couldn't acquire gl context!");

    println!("Window created but hidden!");
    println!("OpenGL Version: {}", gl_wrapper::get_gl_version_str());

    // Create GLSL shaders
    println!("Loading shaders ...");
    let mut program = {
        // Program and shader provide their own error messages
        let vs = shader::VertexShader::new(VS_SRC).unwrap();
        let fs = shader::FragmentShader::new(FS_SRC).unwrap();
        program::Program::new(&[&vs.into(), &fs.into()]).unwrap()
    };

    program.bind_program();
    // TODO: Add an auto_load_all() function
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

    // NOTE: "with_data" constructors grantee that the object crated WILL be bound "new" constructors do not
    // Load textures
    println!("Loading textures ...");

    let t = {
        let im = image::open(&Path::new("apple.png"))
            .expect("Failed to read texture from disk! Are you sure it exists?")
            .into_rgba();
        texture::Texture2D::with_data([im.width(), im.height()], im.as_ref(), gl::RGBA)
            .expect("Failed to crate texture!")
    };
    t.bind_texture_for_sampling(program.get_sampler_id("obj_tex"));

    println!("Done!");

    // Load mesh data ( indices, vertices, uv data )
    println!("Loading mesh ...");
    let mut a = aggregator_obj::VAO::new();
    a.bind_vao_for_data();

    let pos_vbo = buffer_obj::VBO::<GLfloat>::with_data(&[2], &VERTEX_DATA, gl::STATIC_DRAW)
        .expect("Failed to upload data to vbo!");
    a.attach_bound_vbo_to_bound_vao(&pos_vbo, program.get_attribute_id("position"), 0)
        .expect("Failed to attach vob to vao!");

    let tex_vbo = buffer_obj::VBO::<GLfloat>::with_data(&[2], &TEX_DATA, gl::STATIC_DRAW)
        .expect("Failed to upload data to vbo!");
    a.attach_bound_vbo_to_bound_vao(&tex_vbo, program.get_attribute_id("tex_coord"), 0)
        .expect("Failed to attach vbo to vao!");

    a.bind_vao_for_program(&program).expect("Shader is asking for more values than vao has attached, all attributes the shader uses must be attached to vao!");

    let ind_ibo = buffer_obj::IBO::<GLushort>::with_data(&IND_DATA, gl::STATIC_DRAW)
        .expect("Failed to create ibo!");
    println!("Done!");

    println!("Showing window!");
    gl_window.window().set_visible(true);

    gl_wrapper::set_gl_clear_color(0.0, 0.0, 1.0, 1.0);
    // Since these values won't change and the gl::DrawElements is in the hot path we are going to cache these values now just to make things simpler and faster
    let ibo_len = ind_ibo
        .get_size()
        .try_into()
        .expect("The number of triangles you have is either negative, or too big!");
    let mut t = Instant::now();
    // Note we use run-return to make sure that everything gets dropped ( although run also works )
    events_loop.run_return(|event, _, control_flow| {
        // Unless we rewrite the control flow just wait until another evetn arrives when this iteration finished
        *control_flow = ControlFlow::Poll;
        match event {
            // Window stuff
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    gl_wrapper::set_gl_draw_size(width, height);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        glutin::event::KeyboardInput {
                            virtual_keycode: Some(glutin::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            // Rendering stuff
            Event::RedrawEventsCleared => {
                // Lock FPS to 60
                if 1.0 / (t.elapsed().as_secs_f32()) < 61.0 {
                    unsafe {
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                        gl::DrawElements(
                            gl::TRIANGLES,
                            ibo_len,
                            GLushort::get_gl_enum(),
                            ptr::null(),
                        );
                    }
                    gl_window.swap_buffers().unwrap();
                    t = Instant::now();
                }
            }
            _ => {}
        }
    });
}
