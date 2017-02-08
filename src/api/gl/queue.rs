// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::gl::profile_event::PROFILE_EVENT_FUNCTIONS;
use api::gl::sync_event::SYNC_EVENT_FUNCTIONS;
use buffer::{Buffer, Protection};
use error::Error;
use gl::types::{GLint, GLuint};
use gl;
use image::{Color, Image};
use profile_event::ProfileEvent;
use program::Program;
use queue::{Queue, QueueFunctions, Uniform};
use std::os::raw::c_void;
use sync_event::SyncEvent;

pub static QUEUE_FUNCTIONS: QueueFunctions = QueueFunctions {
    destroy: destroy,
    flush: flush,
    finish: finish,
    submit_compute: submit_compute,
    submit_clear: submit_clear,
    submit_read_buffer: submit_read_buffer,
    submit_sync_event: submit_sync_event,
};

unsafe fn destroy(_: &Queue) {}

fn flush(_: &Queue) -> Result<(), Error> {
    unsafe {
        gl::Flush();
        Ok(())
    }
}

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
                  _: &[SyncEvent])
                  -> Result<ProfileEvent, Error> {
    unsafe {
        gl::UseProgram(program.data() as GLuint);

        for &(uniform_index, ref uniform) in uniforms {
            match *uniform {
                Uniform::Buffer(buffer) => {
                    gl::MemoryBarrier(gl::SHADER_STORAGE_BARRIER_BIT);

                    let mut buffer_size = 0;
                    gl::BindBuffer(gl::COPY_READ_BUFFER, buffer.data() as u32);
                    gl::GetBufferParameteriv(gl::COPY_READ_BUFFER,
                                             gl::BUFFER_SIZE,
                                             &mut buffer_size);

                    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER,
                                       uniform_index,
                                       buffer.data() as GLuint);
                }
                Uniform::Image(image) => {
                    gl::MemoryBarrier(gl::TEXTURE_FETCH_BARRIER_BIT |
                                      gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

                    let access = match image.data()[1] {
                        p if p == Protection::ReadOnly as usize => gl::READ_ONLY,
                        p if p == Protection::WriteOnly as usize => gl::WRITE_ONLY,
                        _ => gl::READ_WRITE,
                    };

                    let mut internal_format = 0;
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_RECTANGLE, image.data()[0] as GLuint);
                    gl::GetTexLevelParameteriv(gl::TEXTURE_RECTANGLE,
                                               0,
                                               gl::TEXTURE_INTERNAL_FORMAT,
                                               &mut internal_format);
                    gl::BindTexture(gl::TEXTURE_RECTANGLE, 0);

                    gl::BindImageTexture(uniform_index,
                                         image.data()[0] as GLuint,
                                         0,
                                         gl::FALSE,
                                         0,
                                         access,
                                         internal_format as GLuint);
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

        let mut query = 0;
        gl::GenQueries(1, &mut query);
        gl::BeginQuery(gl::TIME_ELAPSED, query);

        gl::DispatchCompute(*num_groups.get(0).unwrap_or(&1),
                            *num_groups.get(1).unwrap_or(&1),
                            *num_groups.get(2).unwrap_or(&1));

        gl::EndQuery(gl::TIME_ELAPSED);

        Ok(ProfileEvent::from_raw_data(query as usize, &PROFILE_EVENT_FUNCTIONS))
    }
}

fn submit_clear(_: &Queue, image: &Image, color: &Color, _: &[SyncEvent])
                -> Result<ProfileEvent, Error> {
    unsafe {
        let color = match *color {
            Color::UInt(r, _, _, _) => r as u8,
        };

        let mut query = 0;
        gl::GenQueries(1, &mut query);
        gl::BeginQuery(gl::TIME_ELAPSED, query);

        gl::ClearTexImage(image.data()[0] as GLuint,
                          0,
                          gl::RED,
                          gl::UNSIGNED_BYTE,
                          &color as *const u8 as *const c_void);

        gl::EndQuery(gl::TIME_ELAPSED);

        Ok(ProfileEvent::from_raw_data(query as usize, &PROFILE_EVENT_FUNCTIONS))
    }
}

fn submit_read_buffer(_: &Queue, dest: &mut [u8], buffer: &Buffer, start: usize, _: &[SyncEvent])
                      -> Result<ProfileEvent, Error> {
    unsafe {
        let mut query = 0;
        gl::GenQueries(1, &mut query);
        gl::BeginQuery(gl::TIME_ELAPSED, query);

        gl::BindBuffer(gl::COPY_READ_BUFFER, buffer.data() as GLuint);
        gl::GetBufferSubData(gl::COPY_READ_BUFFER,
                             start as isize,
                             dest.len() as isize,
                             dest.as_mut_ptr() as *mut c_void);

        gl::EndQuery(gl::TIME_ELAPSED);

        Ok(ProfileEvent::from_raw_data(query as usize, &PROFILE_EVENT_FUNCTIONS))
    }
}

fn submit_sync_event(_: &Queue) -> Result<SyncEvent, Error> {
    unsafe {
        let fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        Ok(SyncEvent::from_raw_data(fence as usize, &SYNC_EVENT_FUNCTIONS))
    }
}

