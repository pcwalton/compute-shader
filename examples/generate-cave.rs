/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

extern crate compute_shader;
extern crate euclid;
extern crate gl;
extern crate glfw;
extern crate rand;

use compute_shader::buffer::{BufferData, Protection};
use compute_shader::instance::{Instance, ShadingLanguage};
use compute_shader::queue::Uniform;
use compute_shader::texture::ExternalTexture;
use euclid::Size2D;
use gl::types::{GLint, GLsizei, GLuint, GLvoid};
use glfw::{Action, Context, Key, OpenGlProfileHint, Window, WindowEvent, WindowHint, WindowMode};
use rand::Rng;
use std::mem;
use std::os::raw::c_void;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const ITERATIONS: u32 = 128;

#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [i32; 2],
}

// A simple cave generator.
//
// See: http://bit.ly/2hWytfH
pub fn main() {
    let mut glfw = glfw::init(glfw::LOG_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    let context = glfw.create_window(WIDTH, HEIGHT, "generate-cave", WindowMode::Windowed);

    let (mut window, events) = context.expect("Couldn't create a window!");
    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const c_void);

    let instance = Instance::new().unwrap();
    let device = instance.create_device().unwrap();

    let source = match instance.shading_language() {
        ShadingLanguage::Cl => CL_SHADER,
        ShadingLanguage::Glsl => panic!(),
    };
    let program = device.create_program(source).unwrap();

    let buffer_data = BufferData::Uninitialized(WIDTH as usize * HEIGHT as usize);
    let buffer = device.create_buffer(Protection::ReadWrite, buffer_data).unwrap();
    let dest = device.create_texture(Protection::WriteOnly, &Size2D::new(WIDTH, HEIGHT)).unwrap();
    let seed: u32 = rand::thread_rng().next_u32();

    let mut texture = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        dest.bind_to(&ExternalTexture::Gl(texture)).unwrap();
    }

    let groups = [WIDTH, HEIGHT];
    let uniforms = [
        (0, Uniform::Texture(&dest)),
        (1, Uniform::Buffer(&buffer)),
        (2, Uniform::U32(seed)),
        (3, Uniform::U32(ITERATIONS)),
    ];
    let queue = device.create_queue().unwrap();
    queue.submit_compute(&program, &groups, &uniforms, &[]).unwrap().wait().unwrap();

    blit_texture_to_screen(&mut window, texture);

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        }
    }
}

fn blit_texture_to_screen(window: &mut Window, texture: GLuint) {
    unsafe {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER); glck();
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER); glck();
        gl::ShaderSource(vertex_shader,
                         1,
                         &(VERTEX_SHADER.as_ptr() as *const u8 as *const i8),
                         &(VERTEX_SHADER.len() as i32)); glck();
        gl::ShaderSource(fragment_shader,
                         1,
                         &(FRAGMENT_SHADER.as_ptr() as *const u8 as *const i8),
                         &(FRAGMENT_SHADER.len() as i32)); glck();
        gl::CompileShader(vertex_shader); glck();
        gl::CompileShader(fragment_shader); glck();

        let program = gl::CreateProgram(); glck();
        gl::AttachShader(program, vertex_shader); glck();
        gl::AttachShader(program, fragment_shader); glck();
        gl::LinkProgram(program); glck();

        let mut vertex_array = 0;
        gl::GenVertexArrays(1, &mut vertex_array); glck();
        gl::BindVertexArray(vertex_array); glck();
        gl::UseProgram(program); glck();

        let texture_uniform = gl::GetUniformLocation(program,
                                                     b"uTexture\0" as *const u8 as *const i8);
        let position_attrib = gl::GetAttribLocation(program,
                                                    b"aPosition\0" as *const u8 as *const i8);
        let tex_coord_attrib = gl::GetAttribLocation(program,
                                                     b"aTexCoord\0" as *const u8 as *const i8);

        let (mut texture_width, mut texture_height) = (0, 0);
        gl::ActiveTexture(gl::TEXTURE0); glck();
        gl::BindTexture(gl::TEXTURE_RECTANGLE, texture); glck();
        gl::GetTexLevelParameteriv(gl::TEXTURE_RECTANGLE,
                                   0,
                                   gl::TEXTURE_WIDTH,
                                   &mut texture_width);
        gl::GetTexLevelParameteriv(gl::TEXTURE_RECTANGLE,
                                   0,
                                   gl::TEXTURE_HEIGHT,
                                   &mut texture_height);

        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

        let vertices = [
            Vertex { position: [-1.0,  1.0], tex_coord: [            0, texture_height] },
            Vertex { position: [ 1.0,  1.0], tex_coord: [texture_width, texture_height] },
            Vertex { position: [-1.0, -1.0], tex_coord: [            0,              0] },
            Vertex { position: [ 1.0, -1.0], tex_coord: [texture_width,              0] },
        ];

        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer); glck();
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer); glck();
        gl::BufferData(gl::ARRAY_BUFFER,
                       mem::size_of::<[Vertex; 4]>() as isize,
                       &vertices as *const Vertex as *const c_void,
                       gl::STATIC_DRAW); glck();

        gl::VertexAttribPointer(position_attrib as GLuint,
                                2,
                                gl::FLOAT,
                                gl::FALSE,
                                mem::size_of::<Vertex>() as GLsizei,
                                0 as *const GLvoid); glck();
        gl::VertexAttribPointer(tex_coord_attrib as GLuint,
                                2,
                                gl::INT,
                                gl::FALSE,
                                mem::size_of::<Vertex>() as GLsizei,
                                8 as *const GLvoid); glck();
        gl::EnableVertexAttribArray(position_attrib as GLuint); glck();
        gl::EnableVertexAttribArray(tex_coord_attrib as GLuint); glck();

        gl::Uniform1i(texture_uniform, 0); glck();

        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4); glck();

        window.swap_buffers();
    }
}

#[cfg(debug_assertions)]
fn glck() {
    unsafe {
        assert_eq!(gl::GetError(), gl::NO_ERROR);
    }
}

#[cfg(not(debug_assertions))]
fn glck() {}

static CL_SHADER: &'static str = r#"
    // Xorshift32
    uint rand(uint state) {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        return state;
    }

    uint offset(int2 pos, int2 dimensions) {
        return (uint)pos.y * dimensions.x + (uint)pos.x;
    }

    uchar value(bool on) {
        return on ? 255 : 0;
    }

    uint countNeighbors(__global uchar *buffer, int2 p, int2 dimensions) {
        uint neighbors = 0;
        for (int y = p.y - 1; y <= p.y + 1; y++) {
            if (y >= 0 && y < dimensions.y) {
                for (int x = p.x - 1; x <= p.x + 1; x++) {
                    if (x >= 0 && x < dimensions.x && (y != p.y || x != p.x)) {
                        if (buffer[offset((int2)(x, y), dimensions)] != 0)
                            neighbors++;
                    }
                }
            }
        }
        return neighbors;
    }

    __kernel void generate_caves(__write_only image2d_t gTexture,
                                 __global uchar *buffer,
                                 uint kSeed,
                                 uint kIterations) {
        // Based on xxHash
        uint state = kSeed;
        state *= get_global_id(0);
        state ^= state >> 13;
        state *= get_global_id(1);
        state ^= state >> 16;

        // Initial state
        int2 dimensions = get_image_dim(gTexture);
        int2 home = (int2)((int)get_global_id(0), (int)get_global_id(1));
        bool on = rand(state) < 0x73333333;
        buffer[offset(home, dimensions)] = value(on);

        for (uint i = 0; i < kIterations; i++) {
            barrier(CLK_GLOBAL_MEM_FENCE);
            uint neighbors = countNeighbors(buffer, home, dimensions);

            // Verbosity to work around an LLVM bug.
            if (on && neighbors < 3)
                on = false;
            else if (neighbors > 4)
                on = true;

            barrier(CLK_GLOBAL_MEM_FENCE);
            buffer[offset(home, dimensions)] = value(on);
        }

        uint4 color = (uint4)((uint)value(on), (uint)value(on), (uint)value(on), (uint)value(on));
        write_imageui(gTexture, home, color);
    }
"#;

static VERTEX_SHADER: &'static str = r#"
    #version 330

    in vec2 aPosition;
    in vec2 aTexCoord;

    out vec2 vTexCoord;

    void main() {
        vTexCoord = aTexCoord;
        gl_Position = vec4(aPosition, 0.0f, 1.0f);
    }
"#;

static FRAGMENT_SHADER: &'static str = r#"
    #version 330

    uniform sampler2DRect uTexture;

    in vec2 vTexCoord;

    out vec4 oFragColor;

    void main() {
        oFragColor = vec4(texture(uTexture, vTexCoord).rrr * 0.25, 1.0);
    }
"#;

