/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

extern crate byteorder;
extern crate compute_shader;
extern crate gl;
extern crate glfw;
extern crate rand;

use byteorder::{NativeEndian, ReadBytesExt};
use compute_shader::buffer::{BufferData, HostAllocatedData, Protection};
use compute_shader::instance::{Instance, ShadingLanguage};
use compute_shader::queue::Uniform;
use glfw::{Context, WindowHint, WindowMode};
use rand::Rng;
use std::env;
use std::io::Cursor;
use std::mem;
use std::os::raw::c_void;

const DEFAULT_MATRIX_LENGTH: usize = 512;

// Naïve matrix multiplication, just as a demo.
//
// With an unsupported GPU on Linux (e.g. VM), try:
//
//     $ LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=softpipe ./matrix-multiply 16
//
// (Because the shader code is totally naïve, this will run slower on the GPU than on the CPU.
// Don't use the shader code for anything.)
pub fn main() {
    let matrix_length = env::args().nth(1)
                                   .and_then(|arg| arg.parse().ok())
                                   .unwrap_or(DEFAULT_MATRIX_LENGTH);

    let mut glfw = glfw::init(glfw::LOG_ERRORS).unwrap();
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    glfw.window_hint(WindowHint::Visible(false));
    let mut context = glfw.create_window(320, 240, "matrix-multiply", WindowMode::Windowed);
    if context.is_none() {
        // This branch triggers on macOS…
        glfw.window_hint(WindowHint::ContextVersion(1, 0));
        context = glfw.create_window(320, 240, "matrix-multiply", WindowMode::Windowed);
    }

    let mut window = context.expect("Couldn't create a GLFW window!").0;
    window.make_current();
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const c_void);

    let instance = Instance::new().unwrap();
    let device = instance.create_device().unwrap();

    let source = match instance.shading_language() {
        ShadingLanguage::Cl => CL_SHADER,
        ShadingLanguage::Glsl => GL_SHADER,
    };
    let program = device.create_program(source).unwrap();

    let mut thread_rng = rand::thread_rng();

    let input: Vec<f32> = thread_rng.gen_iter().take(matrix_length * matrix_length).collect();
    println!("Input:");
    print(&input, matrix_length);

    let input = BufferData::HostAllocated(HostAllocatedData::new(&input));
    let input = device.create_buffer(Protection::ReadOnly, input).unwrap();
    let output = BufferData::Uninitialized(matrix_length * matrix_length * mem::size_of::<f32>());
    let output = device.create_buffer(Protection::WriteOnly, output).unwrap();

    let queue = device.create_queue().unwrap();
    let groups = [matrix_length as u32, matrix_length as u32];
    let uniforms = [
        (0, Uniform::Buffer(&output)),
        (1, Uniform::Buffer(&input)),
        (2, Uniform::U32(matrix_length as u32)),
    ];
    let event = queue.submit_compute(&program, &groups, &uniforms, &[]).unwrap();

    let mut result_bytes = vec![0; matrix_length * matrix_length * mem::size_of::<f32>()];
    queue.submit_read_buffer(&mut result_bytes, &output, 0, &[event]).unwrap().wait().unwrap();

    let mut result = Vec::with_capacity(matrix_length * matrix_length);
    let mut result_cursor = Cursor::new(result_bytes);
    while let Ok(value) = result_cursor.read_f32::<NativeEndian>() {
        result.push(value)
    }

    println!("\nResult:");
    print(&result, matrix_length);
}

fn print(matrix: &[f32], matrix_length: usize) {
    for row in 0..matrix_length {
        for column in 0..matrix_length {
            print!("{} ", matrix[row * matrix_length + column])
        }
        println!("");
    }
}

static CL_SHADER: &'static str = r#"
    __kernel void matrix_multiply(__global __write_only float *gOutput,
                                  __global __read_only float *gInput,
                                  uint kLength) {
        uint destColumn = get_global_id(0), destRow = get_global_id(1);
        float value = 0.0f;
        for (uint i = 0; i < kLength; i++)
            value += gInput[i * kLength + destColumn] * gInput[destRow * kLength + i];
        gOutput[destRow * kLength + destColumn] = value;
    }
"#;

static GL_SHADER: &'static str = r#"
    #version 330
    #extension GL_ARB_compute_shader : require
    #extension GL_ARB_explicit_uniform_location : require
    #extension GL_ARB_shader_storage_buffer_object : require

    layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

    layout(std430, binding = 0) buffer ssboOutput {
        float gOutput[];
    };
    layout(std430, binding = 1) buffer ssboInput {
        float gInput[];
    };
    layout(location = 2) uniform uint uLength;

    void main() {
        uint destColumn = gl_GlobalInvocationID[0], destRow = gl_GlobalInvocationID[1];
        float value = 0.0f;
        for (uint i = 0u; i < uLength; i++)
            value += gInput[i * uLength + destColumn] * gInput[destRow * uLength + i];
        gOutput[destRow * uLength + destColumn] = value;
    }
"#;

