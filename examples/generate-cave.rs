/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

extern crate compute_shader;
extern crate euclid;
extern crate gl;
extern crate glfw;
extern crate lord_drawquaad;
extern crate rand;

use compute_shader::buffer::{BufferData, Protection};
use compute_shader::image::{ExternalImage, Format};
use compute_shader::instance::{Instance, ShadingLanguage};
use compute_shader::queue::Uniform;
use euclid::Size2D;
use gl::types::GLint;
use glfw::{Action, Context, Key, OpenGlProfileHint, WindowEvent};
use glfw::{WindowHint, WindowMode};
use rand::Rng;
use std::mem;
use std::os::raw::c_void;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const ITERATIONS: u32 = 8;

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
    let device = instance.open_device().unwrap();

    let source = match instance.shading_language() {
        ShadingLanguage::Cl => CL_SHADER,
        ShadingLanguage::Glsl => GL_SHADER,
    };
    let program = device.create_program(source).unwrap();

    let draw_context = lord_drawquaad::Context::new();

    let buffer_data = BufferData::Uninitialized(WIDTH as usize * HEIGHT as usize *
                                                mem::size_of::<f32>());
    let buffer = device.create_buffer(Protection::ReadWrite, buffer_data).unwrap();
    let dest = device.create_image(Format::R8, Protection::ReadWrite, &Size2D::new(WIDTH, HEIGHT))
                     .unwrap();
    let seed: u32 = rand::thread_rng().next_u32();

    let mut texture = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        dest.bind_to(&ExternalImage::GlTexture(texture)).unwrap();

        gl::BindTexture(gl::TEXTURE_RECTANGLE, texture);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_RECTANGLE, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
    }

    let groups = [WIDTH, HEIGHT];
    let uniforms = [
        (0, Uniform::Image(&dest)),
        (1, Uniform::Buffer(&buffer)),
        (2, Uniform::U32(seed)),
        (3, Uniform::U32(ITERATIONS)),
    ];
    let queue = device.create_queue().unwrap();
    queue.submit_compute(&program, &groups, &uniforms, &[]).unwrap();
    queue.submit_sync_event().unwrap().wait().unwrap();

    unsafe {
        gl::MemoryBarrier(gl::TEXTURE_FETCH_BARRIER_BIT | gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    draw_context.draw(texture);
    window.swap_buffers();

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

    __kernel void generate_caves(__write_only image2d_t gImage,
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
        int2 dimensions = get_image_dim(gImage);
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
        write_imageui(gImage, home, color);
    }
"#;

static GL_SHADER: &'static str = r#"
    #version 330
    #extension GL_ARB_compute_shader : require
    #extension GL_ARB_explicit_uniform_location : require
    #extension GL_ARB_shader_storage_buffer_object : require
    #extension GL_ARB_shader_image_load_store : require
    #extension GL_ARB_shader_image_size : require
    #extension GL_ARB_shading_language_420pack : require

    layout(r8, binding = 0) uniform restrict writeonly image2D uImage;

    layout(std430, binding = 1) buffer ssbBuffer {
        float gBuffer[480000];
    };

    layout(location = 2) uniform uint uSeed;
    layout(location = 3) uniform uint uIterations;

    layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

    // Xorshift32
    uint rand(uint state) {
        state ^= state << 13u;
        state ^= state >> 17u;
        state ^= state << 5u;
        return state;
    }

    uint offset(ivec2 pos, ivec2 dimensions) {
        return uint(pos.y) * uint(dimensions.x) + uint(pos.x);
    }

    float value(bool on) {
        return on ? 1.0 : 0.0;
    }

    uint countNeighbors(ivec2 p, ivec2 dimensions) {
        uint neighbors = 0u;
        for (int y = p.y - 1; y <= p.y + 1; y++) {
            if (y >= 0 && y < dimensions.y) {
                for (int x = p.x - 1; x <= p.x + 1; x++) {
                    if (x >= 0 && x < dimensions.x && (y != p.y || x != p.x)) {
                        if (gBuffer[offset(ivec2(x, y), dimensions)] != 0.0)
                            neighbors++;
                    }
                }
            }
        }
        return neighbors;
    }

    void main() {
        // Based on xxHash
        uint state = uSeed;
        state *= uint(gl_GlobalInvocationID.x);
        state ^= state >> 13u;
        state *= uint(gl_GlobalInvocationID.y);
        state ^= state >> 16u;

        // Initial state
        ivec2 dimensions = imageSize(uImage);
        ivec2 home = ivec2(gl_GlobalInvocationID.xy);
        bool inBounds = home.x < dimensions.x && home.y < dimensions.y;
        bool on = rand(state) < 0x73333333u;
        if (inBounds)
            gBuffer[offset(home, dimensions)] = value(on);

        for (uint i = 0u; i < uIterations; i++) {
            barrier();
            uint neighbors = inBounds ? countNeighbors(home, dimensions) : 0u;

            if (on && neighbors < 3u)
                on = false;
            else if (neighbors > 4u)
                on = true;

            barrier();
            if (inBounds)
                gBuffer[offset(home, dimensions)] = value(on);
        }

        vec4 color = vec4(value(on));
        if (inBounds)
            imageStore(uImage, home, color);
    }
"#;

