extern crate gl_wrapper;
extern crate glutin;
extern crate image;

use gl_wrapper::render::{program, shader, texture};
use gl_wrapper::util::{buffer_obj, aggregator_obj};
use gl_wrapper::util::buffer_obj::BOFunc;
use gl_wrapper::HasGLEnum;

use glutin::dpi::PhysicalSize;
use std::convert::TryInto;

use gl::types::*;
use std::ptr;
use std::str;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;

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
    let mut vbo_bouncer = buffer_obj::VBOBouncer::new();
    let mut ibo_bouncer = buffer_obj::IBOBouncer::new();
    let mut prog_bouncer = program::ProgramBouncer::new();

    let mut vao_bouncer = aggregator_obj::VAOBouncer::new();


    let events_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize {
            width: 400.0,
            height: 400.0,
        })
        .with_visible(false); // Hide window while loading to make it less annoying
    let gl_window = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window, &events_loop)
        .expect("Creating window!");

    // Load the OpenGL function pointers
    let gl_window = gl_wrapper::init(gl_window).expect("Acquiring gl context!");
    let mut max_combined_texture_image_units: GLint = 0;

    unsafe {
        gl::GetIntegerv(
            gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS,
            &mut max_combined_texture_image_units,
        );
    }
    println!("Units: {}", max_combined_texture_image_units);

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

    let mut program = program.bind_mut(&mut prog_bouncer);
    program.load_attribute("position").expect("Loading attribute 'position'");
    program.load_attribute("tex_coord").expect("Loading attribute 'tex_coord'");
    program.load_uniform("obj_tex").expect("Loading uniform sampler 'obj_tex'");

    const TUNIT: usize = 0;
    { let id = program.get_uniform_id("obj_tex").unwrap().try_into().unwrap(); program.set_uniform_i32(id, TUNIT.try_into().unwrap()); }
    let mut texunit_bouncer = texture::TextureBouncer::<TUNIT>::new();

    println!("Done!");

    // NOTE: "with_data" constructors grantee that the object crated WILL be bound "new" constructors do not
    // Load textures
    println!("Loading textures ...");

    let t = {
        let im = image::open(&Path::new("apple.png"))
            .expect("Reading textures!")
            .into_rgba8();
        texture::Texture2D::with_data(
            &mut texunit_bouncer,
            [
                im.width().try_into().unwrap(),
                im.height().try_into().unwrap(),
            ],
            im.as_ref(),
            gl::RGBA,
        )
        .expect("Creating textures.")
    };
    let texid: usize = program.get_sampler_id("obj_tex").unwrap() as usize;
    assert_eq!(texid, 0);
    let _t = t.bind(&mut texunit_bouncer);

    println!("Done!");

    // Load mesh data ( indices, vertices, uv data )
    println!("Loading mesh ...");
    let mut a = aggregator_obj::VAO::new();
    let mut a = a.bind_mut( &mut vao_bouncer);

    let pos_vbo = buffer_obj::VBO::<GLfloat>::with_data(
        &mut vbo_bouncer,
        &[2],
        &VERTEX_DATA,
        gl::STATIC_DRAW,
    )
    .expect("Uploading pos data to vbo!");

    let pos_vbo = pos_vbo.bind(&mut vbo_bouncer);
    a.attach_vbo_to_vao(
        &pos_vbo,
        program.get_attribute_id("position").unwrap(),
        0,
        false,
    )
    .expect("Attaching pos vbo to vao!");

    let tex_vbo =
        buffer_obj::VBO::<GLfloat>::with_data(&mut vbo_bouncer, &[2], &TEX_DATA, gl::STATIC_DRAW)
            .expect("Uploading tex data to vbo!");

    let tex_vbo = tex_vbo.bind(&mut vbo_bouncer);
    a.attach_vbo_to_vao(
        &tex_vbo,
        program.get_attribute_id("tex_coord").unwrap(),
        0,
        false,
    )
    .expect("Attaching tex vbo to vao!");

    a.adapt_vao_to_program(&program)
        .expect("Linking shader attributes to vao data!");

    let ind_ibo = buffer_obj::IBO::<GLushort>::with_data(&mut ibo_bouncer, &IND_DATA, gl::STATIC_DRAW)
        .expect("Uploading indecies to ibo!");
    let ind_ibo = ind_ibo.bind( &mut ibo_bouncer);
    println!("Done!");

    println!("Showing window!");
    gl_window.window().set_visible(true);

    gl_wrapper::set_gl_clear_color(0.0, 0.0, 1.0, 1.0).unwrap();

    // Since these values won't change and the gl::DrawElements is in the hot path we are going to cache these values now just to make things simpler and faster
    let ibo_len = ind_ibo
        .get_size()
        .try_into()
        .expect("Getting number of triangles!");

    let mut t = Instant::now();
    // Note we use run-return to make sure that everything gets dropped ( although run also works )
    events_loop.run(move |event, _, control_flow| {
        // Set default for control_flow
        *control_flow = ControlFlow::Poll;
        match event {
            // Window stuff
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    gl_wrapper::set_gl_draw_size(width, height).unwrap();
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                // Handle esc key
                WindowEvent::KeyboardInput {
                    input:
                        glutin::event::KeyboardInput {
                            virtual_keycode: Some(glutin::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,

                _ => {} // Default
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
                            GLushort::get_gl_type(),
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
