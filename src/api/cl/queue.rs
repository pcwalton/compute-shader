// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::cl::ffi::{self, CL_IMAGE_DEPTH, CL_IMAGE_HEIGHT, CL_IMAGE_WIDTH, CL_SUCCESS, CL_TRUE};
use api::cl::ffi::{cl_command_queue, cl_event, cl_mem, cl_program};
use api::cl::event::EVENT_FUNCTIONS;
use buffer::Buffer;
use error::Error;
use event::Event;
use program::Program;
use queue::{Queue, QueueFunctions, Uniform};
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use texture::{Color, Texture};

pub static QUEUE_FUNCTIONS: QueueFunctions = QueueFunctions {
    destroy: destroy,
    finish: finish,
    submit_compute: submit_compute,
    submit_clear: submit_clear,
    submit_read_buffer: submit_read_buffer,
};

unsafe fn destroy(this: &Queue) {
    ffi::clReleaseCommandQueue(this.data as cl_command_queue);
}

fn finish(this: &Queue) -> Result<(), Error> {
    unsafe {
        if ffi::clFinish(this.data as cl_command_queue) == CL_SUCCESS {
            Ok(())
        } else {
            Err(Error::Failed)
        }
    }
}

fn submit_compute(this: &Queue,
                  program: &Program,
                  num_groups: &[u32],
                  uniforms: &[(u32, Uniform)],
                  events: &[Event])
                  -> Result<Event, Error> {
    unsafe {
        let mut kernel = ptr::null_mut();
        if ffi::clCreateKernelsInProgram(program.data as cl_program,
                                         1,
                                         &mut kernel,
                                         ptr::null_mut()) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        for &(uniform_index, ref uniform) in uniforms {
            let (arg_size, arg_value);
            match *uniform {
                Uniform::Buffer(buffer) => {
                    arg_size = mem::size_of::<cl_mem>();
                    arg_value = &buffer.data as *const usize as *const c_void
                }
                Uniform::Texture(texture) => {
                    arg_size = mem::size_of::<cl_mem>();
                    arg_value = &texture.data[0] as *const usize as *const c_void
                }
                Uniform::U32(ref value) => {
                    arg_size = mem::size_of::<u32>();
                    arg_value = value as *const u32 as *const c_void
                }
                Uniform::UVec4(ref value) => {
                    arg_size = mem::size_of::<u32>();
                    arg_value = &value[0] as *const u32 as *const c_void
                }
            }

            if ffi::clSetKernelArg(kernel, uniform_index, arg_size, arg_value) != CL_SUCCESS {
                return Err(Error::Failed)
            }
        }

        let mut global_work_size = [0; 3];
        for (dimension, &group_size) in num_groups.iter().enumerate() {
            global_work_size[dimension] = group_size as usize
        }

        let event_wait_list: Vec<_> = events.iter().map(|event| event.data as cl_event).collect();
        let event_wait_list_ptr = if event_wait_list.is_empty() {
            ptr::null()
        } else {
            event_wait_list.as_ptr()
        };

        let mut event = ptr::null_mut();

        if ffi::clEnqueueNDRangeKernel(this.data as cl_command_queue,
                                       kernel,
                                       num_groups.len() as u32,
                                       ptr::null(),
                                       global_work_size.as_mut_ptr(),
                                       ptr::null(),
                                       event_wait_list.len() as u32,
                                       event_wait_list_ptr,
                                       &mut event) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        ffi::clReleaseKernel(kernel);

        Ok(Event {
            data: event as usize,
            functions: &EVENT_FUNCTIONS,
        })
    }
}

fn submit_clear(this: &Queue, texture: &Texture, color: &Color, events: &[Event])
                -> Result<Event, Error> {
    unsafe {
        let colors = match *color {
            Color::UInt(r, g, b, a) => [r, g, b, a],
        };

        let origin = [0, 0, 0];

        let mut size = [0, 0, 0];
        ffi::clGetImageInfo(texture.data[0] as cl_mem,
                            CL_IMAGE_WIDTH,
                            mem::size_of::<usize>(),
                            &mut size[0] as *mut usize as *mut c_void,
                            ptr::null_mut());
        ffi::clGetImageInfo(texture.data[0] as cl_mem,
                            CL_IMAGE_HEIGHT,
                            mem::size_of::<usize>(),
                            &mut size[1] as *mut usize as *mut c_void,
                            ptr::null_mut());
        ffi::clGetImageInfo(texture.data[0] as cl_mem,
                            CL_IMAGE_DEPTH,
                            mem::size_of::<usize>(),
                            &mut size[2] as *mut usize as *mut c_void,
                            ptr::null_mut());
        for length in &mut size {
            if *length == 0 {
                *length = 1
            }
        }

        let event_wait_list: Vec<_> = events.iter().map(|event| event.data as cl_event).collect();
        let event_wait_list_ptr = if event_wait_list.is_empty() {
            ptr::null()
        } else {
            event_wait_list.as_ptr()
        };

        let mut event = ptr::null_mut();

        if ffi::clEnqueueFillImage(this.data as cl_command_queue,
                                   texture.data[0] as cl_mem,
                                   colors.as_ptr() as *const c_void,
                                   origin.as_ptr(),
                                   size.as_mut_ptr(),
                                   event_wait_list.len() as u32,
                                   event_wait_list_ptr,
                                   &mut event) == CL_SUCCESS {
            Ok(Event {
                data: event as usize,
                functions: &EVENT_FUNCTIONS,
            })
        } else {
            Err(Error::Failed)
        }
    }
}

fn submit_read_buffer(this: &Queue,
                      dest: &mut [u8],
                      buffer: &Buffer,
                      start: usize,
                      events: &[Event])
                      -> Result<Event, Error> {
    unsafe {
        let event_wait_list: Vec<_> = events.iter().map(|event| event.data as cl_event).collect();
        let event_wait_list_ptr = if event_wait_list.is_empty() {
            ptr::null()
        } else {
            event_wait_list.as_ptr()
        };

        let mut event = ptr::null_mut();

        if ffi::clEnqueueReadBuffer(this.data as cl_command_queue,
                                    buffer.data as cl_mem,
                                    CL_TRUE,
                                    start,
                                    dest.len(),
                                    dest.as_mut_ptr() as *mut c_void,
                                    event_wait_list.len() as u32,
                                    event_wait_list_ptr,
                                    &mut event) == CL_SUCCESS {
            Ok(Event {
                data: event as usize,
                functions: &EVENT_FUNCTIONS,
            })
        } else {
            Err(Error::Failed)
        }
    }
}

