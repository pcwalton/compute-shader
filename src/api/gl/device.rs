// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::gl::buffer::BUFFER_FUNCTIONS;
use api::gl::program::PROGRAM_FUNCTIONS;
use api::gl::queue::QUEUE_FUNCTIONS;
use api::gl::texture::TEXTURE_FUNCTIONS;
use buffer::{Buffer, BufferData, Protection};
use device::{Device, DeviceFunctions};
use error::Error;
use euclid::Size2D;
use gl::types::GLint;
use gl;
use program::Program;
use queue::Queue;
use std::os::raw::c_void;
use std::ptr;
use texture::{Format, Texture};

pub static DEVICE_FUNCTIONS: DeviceFunctions = DeviceFunctions {
    destroy: destroy,
    create_queue: create_queue,
    create_program: create_program,
    create_buffer: create_buffer,
    create_texture: create_texture,
};

unsafe fn destroy(_: &Device) {}

fn create_queue(_: &Device) -> Result<Queue, Error> {
    Ok(Queue {
        data: 0,
        functions: &QUEUE_FUNCTIONS,
    })
}

fn create_program(_: &Device, source: &str) -> Result<Program, Error> {
    unsafe {
        let shader = gl::CreateShader(gl::COMPUTE_SHADER);
        let mut source_bytes = source.as_ptr() as *const i8;
        let source_length = source.len() as i32;
        gl::ShaderSource(shader, 1, &mut source_bytes, &source_length);
        gl::CompileShader(shader);

        let mut compile_status = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);
        if compile_status != gl::TRUE as GLint {
            let mut info_log_length = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut info_log_length);
            let mut info_log_buffer = vec![0; info_log_length as usize + 1];
            gl::GetShaderInfoLog(shader,
                                 info_log_length,
                                 ptr::null_mut(),
                                 info_log_buffer.as_mut_ptr() as *mut i8);
            let info_log = String::from_utf8(info_log_buffer).unwrap_or("".to_owned());
            return Err(Error::CompileFailed(info_log))
        }

        let program = gl::CreateProgram();
        gl::AttachShader(program, shader);
        gl::LinkProgram(program);

        let mut link_status = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);
        if link_status != gl::TRUE as GLint {
            let mut info_log_length = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut info_log_length);
            let mut info_log_buffer = vec![0; info_log_length as usize + 1];
            gl::GetProgramInfoLog(program,
                                  info_log_length,
                                  ptr::null_mut(),
                                  info_log_buffer.as_mut_ptr() as *mut i8);
            let info_log = String::from_utf8(info_log_buffer).unwrap_or("".to_owned());
            return Err(Error::LinkFailed(info_log))
        }

        Ok(Program {
            data: program as usize,
            functions: &PROGRAM_FUNCTIONS,
        })
    }
}

fn create_buffer(_: &Device, _: Protection, mut data: BufferData) -> Result<Buffer, Error> {
    unsafe {
        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(gl::COPY_WRITE_BUFFER, buffer);

        match data {
            BufferData::HostAllocated(ref mut host_buffer) => {
                gl::BufferData(gl::COPY_WRITE_BUFFER,
                               host_buffer.size() as isize,
                               host_buffer.as_ptr() as *const c_void,
                               gl::DYNAMIC_DRAW)
            }
            BufferData::Uninitialized(size) => {
                gl::BufferData(gl::COPY_WRITE_BUFFER, size as isize, ptr::null(), gl::DYNAMIC_DRAW)
            }
        }

        Ok(Buffer {
            data: buffer as usize,
            functions: &BUFFER_FUNCTIONS,
        })
    }
}

fn create_texture(_: &Device, format: Format, _: Protection, size: &Size2D<u32>)
                  -> Result<Texture, Error> {
    unsafe {
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);

        let gl_format = format.gl_internal_format();
        gl::TexStorage2D(gl::TEXTURE_2D, 0, gl_format, size.width as i32, size.height as i32);

        Ok(Texture {
            data: [texture as usize, 0],
            functions: &TEXTURE_FUNCTIONS,
        })
    }
}

