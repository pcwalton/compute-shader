// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::gl::event::EVENT_FUNCTIONS;
use buffer::Buffer;
use error::Error;
use event::Event;
use gl::types::{GLint, GLuint};
use gl;
use program::Program;
use queue::{Queue, QueueFunctions, Uniform};
use std::os::raw::c_void;
use texture::{Color, Texture};

pub static QUEUE_FUNCTIONS: QueueFunctions = QueueFunctions {
    destroy: destroy,
    finish: finish,
    submit_compute: submit_compute,
    submit_clear: submit_clear,
    submit_read_buffer: submit_read_buffer,
};

unsafe fn destroy(_: &Queue) {}

fn finish(_: &Queue) -> Result<(), Error> {
    unsafe {
        gl::Finish();
        Ok(())
    }
}

fn submit_compute(_: &Queue,
                  program: &Program,
                  num_groups: &[u32],
                  uniforms: &[(u32, Uniform)],
                  _: &[Event])
                  -> Result<Event, Error> {
    unsafe {
        gl::UseProgram(program.data as GLuint);

        let (mut next_ssbo_binding, mut next_texture_unit) = (0, 0);
        for &(uniform_index, ref uniform) in uniforms {
            match *uniform {
                Uniform::Buffer(buffer) => {
                    let mut buffer_size = 0;
                    gl::BindBuffer(gl::COPY_READ_BUFFER, buffer.data as u32);
                    gl::GetBufferParameteriv(gl::COPY_READ_BUFFER,
                                             gl::BUFFER_SIZE,
                                             &mut buffer_size);

                    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER,
                                       next_ssbo_binding,
                                       buffer.data as GLuint);

                    next_ssbo_binding += 1
                }
                Uniform::Texture(texture) => {
                    gl::ActiveTexture(gl::TEXTURE0 + next_texture_unit);
                    gl::BindTexture(gl::TEXTURE_2D, texture.data[0] as GLuint);
                    gl::Uniform1i(uniform_index as GLint, next_texture_unit as GLint);
                    next_texture_unit += 1
                }
                Uniform::U32(value) => gl::Uniform1ui(uniform_index as GLint, value),
                Uniform::UVec4(values) => {
                    gl::Uniform4ui(uniform_index as GLint,
                                   values[0],
                                   values[1],
                                   values[2],
                                   values[3])
                }
            }
        }

        gl::DispatchCompute(*num_groups.get(0).unwrap_or(&1),
                            *num_groups.get(1).unwrap_or(&1),
                            *num_groups.get(2).unwrap_or(&1));

        Ok(Event {
            data: 0,
            functions: &EVENT_FUNCTIONS,
        })
    }
}

fn submit_clear(_: &Queue, texture: &Texture, color: &Color, _: &[Event]) -> Result<Event, Error> {
    unsafe {
        let color = match *color {
            Color::UInt(r, _, _, _) => r as u8,
        };
        gl::ClearTexImage(texture.data[0] as GLuint,
                          0,
                          gl::RED,
                          gl::UNSIGNED_BYTE,
                          &color as *const u8 as *const c_void);

        Ok(Event {
            data: 0,
            functions: &EVENT_FUNCTIONS,
        })
    }
}

fn submit_read_buffer(_: &Queue, dest: &mut [u8], buffer: &Buffer, start: usize, _: &[Event])
                      -> Result<Event, Error> {
    unsafe {
        gl::BindBuffer(gl::COPY_READ_BUFFER, buffer.data as GLuint);
        gl::GetBufferSubData(gl::COPY_READ_BUFFER,
                             start as isize,
                             dest.len() as isize,
                             dest.as_mut_ptr() as *mut c_void);

        Ok(Event {
            data: 0,
            functions: &EVENT_FUNCTIONS,
        })
    }
}

