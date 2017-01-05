/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

extern crate byteorder;
extern crate compute_shader;
extern crate rand;

use byteorder::{NativeEndian, ReadBytesExt};
use compute_shader::buffer::{BufferData, HostAllocatedData, Protection};
use compute_shader::instance::Instance;
use compute_shader::queue::Uniform;
use rand::Rng;
use std::io::Cursor;
use std::mem;

const MATRIX_LENGTH: usize = 1000;

static GROUPS: [u32; 2] = [MATRIX_LENGTH as u32, MATRIX_LENGTH as u32];

// Naïve matrix multiplication, just as a demo.
//
// (Because the shader code is totally naïve, this will run slower on the GPU than on the CPU.
// Don't use the shader code for anything.)
pub fn main() {
    let instance = Instance::new().unwrap();
    let device = instance.create_device().unwrap();
    let program = device.create_program(CL_SHADER).unwrap();

    let mut thread_rng = rand::thread_rng();

    let input: Vec<f32> = thread_rng.gen_iter().take(MATRIX_LENGTH * MATRIX_LENGTH).collect();
    println!("Input:");
    print(&input);

    let input = BufferData::HostAllocated(HostAllocatedData::new(&input));
    let input = device.create_buffer(Protection::ReadOnly, input).unwrap();
    let output = BufferData::Uninitialized(MATRIX_LENGTH * MATRIX_LENGTH * mem::size_of::<f32>());
    let output = device.create_buffer(Protection::WriteOnly, output).unwrap();

    let queue = device.create_queue().unwrap();
    let uniforms = [
        (0, Uniform::Buffer(&output)),
        (1, Uniform::Buffer(&input)),
        (2, Uniform::U32(MATRIX_LENGTH as u32)),
    ];
    let event = queue.submit_compute(&program, &GROUPS, &uniforms, &[]).unwrap();

    let mut result_bytes = vec![0; MATRIX_LENGTH * MATRIX_LENGTH * mem::size_of::<f32>()];
    queue.submit_read_buffer(&mut result_bytes, &output, 0, &[event]).unwrap().wait().unwrap();

    let mut result = Vec::with_capacity(MATRIX_LENGTH * MATRIX_LENGTH);
    let mut result_cursor = Cursor::new(result_bytes);
    while let Ok(value) = result_cursor.read_f32::<NativeEndian>() {
        result.push(value)
    }

    println!("\nResult:");
    print(&result);
}

fn print(matrix: &[f32]) {
    for row in 0..MATRIX_LENGTH {
        for column in 0..MATRIX_LENGTH {
            print!("{} ", matrix[row * MATRIX_LENGTH + column])
        }
        println!("");
    }
}

static CL_SHADER: &'static str = r#"
    __kernel void matrix_multiply(__global __write_only float *gOutput,
                                  __global __read_only float *gInput,
                                  uint length) {
        uint destColumn = get_global_id(0), destRow = get_global_id(1);
        float value = 0.0f;
        for (uint i = 0; i < length; i++)
            value += gInput[i * length + destColumn] * gInput[destRow * length + i];
        gOutput[destRow * length + destColumn] = value;
    }
"#;

