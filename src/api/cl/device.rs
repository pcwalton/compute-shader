// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::cl::buffer::BUFFER_FUNCTIONS;
use api::cl::ffi::{self, CL_CONTEXT_DEVICES, CL_FLOAT, CL_MEM_COPY_HOST_PTR, CL_MEM_READ_ONLY};
use api::cl::ffi::{CL_MEM_READ_WRITE, CL_MEM_WRITE_ONLY, CL_PROGRAM_BUILD_LOG};
use api::cl::ffi::{CL_QUEUE_PROFILING_ENABLE, CL_R, CL_RGBA, CL_SUCCESS, CL_UNORM_INT8};
use api::cl::ffi::{cl_context, cl_device_id, cl_image_format, cl_mem_flags};
use api::cl::image::IMAGE_FUNCTIONS;
use api::cl::program::PROGRAM_FUNCTIONS;
use api::cl::queue::QUEUE_FUNCTIONS;
use buffer::{Buffer, BufferData, Protection};
use device::{Device, DeviceFunctions};
use error::Error;
use euclid::Size2D;
use image::{Format, Image};
use program::Program;
use queue::Queue;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use core_foundation::dictionary::CFDictionary;
#[cfg(target_os = "macos")]
use core_foundation::number::CFNumber;
#[cfg(target_os = "macos")]
use core_foundation::string::CFString;
#[cfg(target_os = "macos")]
use io_surface::{self, kIOSurfaceBytesPerElement, kIOSurfaceHeight, kIOSurfaceWidth};

pub static DEVICE_FUNCTIONS: DeviceFunctions = DeviceFunctions {
    destroy: destroy,
    create_queue: create_queue,
    create_program: create_program,
    create_buffer: create_buffer,
    create_image: create_image,
};

unsafe fn destroy(this: &Device) {
    ffi::clReleaseContext(this.data() as cl_context);
}

fn create_queue(this: &Device) -> Result<Queue, Error> {
    unsafe {
        let mut device_id: cl_device_id = ptr::null_mut();
        if ffi::clGetContextInfo(this.data() as cl_context,
                                 CL_CONTEXT_DEVICES,
                                 mem::size_of::<cl_device_id>(),
                                 &mut device_id as *mut cl_device_id as *mut c_void,
                                 ptr::null_mut()) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        let queue = ffi::clCreateCommandQueue(this.data() as cl_context,
                                              device_id,
                                              CL_QUEUE_PROFILING_ENABLE,
                                              ptr::null_mut());
        if queue != ptr::null_mut() {
            Ok(Queue::from_raw_data(queue as usize, &QUEUE_FUNCTIONS))
        } else {
            Err(Error::Failed)
        }
    }
}

fn create_program(this: &Device, source: &str) -> Result<Program, Error> {
    unsafe {
        let mut strings = source.as_ptr() as *const i8;
        let lengths = source.len();
        let program = ffi::clCreateProgramWithSource(this.data() as cl_context,
                                                     1,
                                                     &mut strings,
                                                     &lengths,
                                                     ptr::null_mut());
        if program == ptr::null_mut() {
            return Err(Error::Failed)
        }

        let mut device_id: cl_device_id = ptr::null_mut();
        if ffi::clGetContextInfo(this.data() as cl_context,
                                 CL_CONTEXT_DEVICES,
                                 mem::size_of::<cl_device_id>(),
                                 &mut device_id as *mut cl_device_id as *mut c_void,
                                 ptr::null_mut()) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        let null = 0;

        if ffi::clBuildProgram(program,
                               1,
                               &device_id,
                               &null,
                               None,
                               ptr::null_mut()) != CL_SUCCESS {
            let mut build_log = vec![0; 65536];
            let mut build_log_size = build_log.len();
            ffi::clGetProgramBuildInfo(program,
                                       device_id,
                                       CL_PROGRAM_BUILD_LOG,
                                       build_log.len() - 1,
                                       build_log.as_mut_ptr() as *mut c_void,
                                       &mut build_log_size);
            build_log.truncate(build_log_size);

            return Err(Error::CompileFailed(String::from_utf8(build_log).unwrap_or("".to_owned())))
        }

        let mut kernel = ptr::null_mut();
        if ffi::clCreateKernelsInProgram(program, 1, &mut kernel, ptr::null_mut()) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        Ok(Program::from_raw_data(kernel as usize, &PROGRAM_FUNCTIONS))
    }
}

fn create_buffer(this: &Device, protection: Protection, mut data: BufferData)
                 -> Result<Buffer, Error> {
    unsafe {
        let mut mem_flags = protection_to_mem_flags(protection);
        let (size, host_ptr);
        match data {
            BufferData::HostAllocated(ref mut buffer) => {
                mem_flags |= CL_MEM_COPY_HOST_PTR;

                size = buffer.size();
                host_ptr = buffer.as_ptr()
            }
            BufferData::Uninitialized(in_size) => {
                size = in_size;
                host_ptr = ptr::null_mut()
            }
        }

        let buffer = ffi::clCreateBuffer(this.data() as cl_context,
                                         mem_flags,
                                         size,
                                         host_ptr as *mut c_void,
                                         ptr::null_mut());
        if !buffer.is_null() {
            Ok(Buffer::from_raw_data(buffer as usize, &BUFFER_FUNCTIONS))
        } else {
            Err(Error::Failed)
        }
    }
}

#[cfg(target_os = "macos")]
fn create_image(this: &Device, format: Format, protection: Protection, size: &Size2D<u32>)
                -> Result<Image, Error> {
    unsafe {
        let bytes_per_element = match format {
            Format::R8 => 1,
            Format::RGBA8 | Format::R32F => 4,
        };

        let properties = CFDictionary::from_CFType_pairs(&[
            (CFString::wrap_under_get_rule(kIOSurfaceWidth),
             CFNumber::from_i64(size.width as i64)),
            (CFString::wrap_under_get_rule(kIOSurfaceHeight),
             CFNumber::from_i64(size.height as i64)),
            (CFString::wrap_under_get_rule(kIOSurfaceBytesPerElement),
             CFNumber::from_i32(bytes_per_element)),
        ]);
        let surface = io_surface::new(&properties);

        let protection = protection_to_mem_flags(protection);

        let image_format = match format {
            Format::R8 => {
                cl_image_format {
                    image_channel_order: CL_R,
                    image_channel_data_type: CL_UNORM_INT8,
                }
            }
            Format::RGBA8 => {
                cl_image_format {
                    image_channel_order: CL_RGBA,
                    image_channel_data_type: CL_UNORM_INT8,
                }
            }
            Format::R32F => {
                cl_image_format {
                    image_channel_order: CL_R,
                    image_channel_data_type: CL_FLOAT,
                }
            }
        };

        let mut error = CL_SUCCESS;

        let image = ffi::clCreateImageFromIOSurface2DAPPLE(this.data() as cl_context,
                                                           protection,
                                                           &image_format,
                                                           size.width as usize,
                                                           size.height as usize,
                                                           surface.as_concrete_TypeRef(),
                                                           &mut error);

        if error != CL_SUCCESS || image.is_null() {
            return Err(Error::Failed)
        }

        let surface_ref = surface.as_concrete_TypeRef();
        mem::forget(surface);

        Ok(Image::from_raw_data([image as usize, surface_ref as usize], &IMAGE_FUNCTIONS))
    }
}

fn protection_to_mem_flags(protection: Protection) -> cl_mem_flags {
    match protection {
        Protection::ReadOnly => CL_MEM_READ_ONLY,
        Protection::WriteOnly => CL_MEM_WRITE_ONLY,
        Protection::ReadWrite => CL_MEM_READ_WRITE,
    }
}

