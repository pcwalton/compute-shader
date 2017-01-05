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
use api::cl::ffi::{self, CL_CONTEXT_DEVICES, CL_MEM_COPY_HOST_PTR, CL_MEM_READ_ONLY};
use api::cl::ffi::{CL_MEM_READ_WRITE, CL_MEM_USE_HOST_PTR, CL_MEM_WRITE_ONLY};
use api::cl::ffi::{CL_QUEUE_PROFILING_ENABLE, CL_R, CL_SUCCESS, CL_UNSIGNED_INT8, cl_context};
use api::cl::ffi::{cl_device_id, cl_image_format, cl_mem_flags};
use api::cl::program::PROGRAM_FUNCTIONS;
use api::cl::queue::QUEUE_FUNCTIONS;
use api::cl::texture::TEXTURE_FUNCTIONS;
use buffer::{Buffer, BufferData, Protection};
use device::{Device, DeviceFunctions};
use error::Error;
use euclid::Size2D;
use program::Program;
use queue::Queue;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use texture::Texture;

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
    create_texture: create_texture,
};

unsafe fn destroy(this: &Device) {
    ffi::clReleaseContext(this.data as cl_context);
}

fn create_queue(this: &Device) -> Result<Queue, Error> {
    unsafe {
        let mut device_id: cl_device_id = ptr::null_mut();
        if ffi::clGetContextInfo(this.data as cl_context,
                                 CL_CONTEXT_DEVICES,
                                 mem::size_of::<cl_device_id>(),
                                 &mut device_id as *mut cl_device_id as *mut c_void,
                                 ptr::null_mut()) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        let queue = ffi::clCreateCommandQueue(this.data as cl_context,
                                              device_id,
                                              CL_QUEUE_PROFILING_ENABLE,
                                              ptr::null_mut());
        if queue != ptr::null_mut() {
            Ok(Queue {
                data: queue as usize,
                functions: &QUEUE_FUNCTIONS,
            })
        } else {
            Err(Error::Failed)
        }
    }
}

fn create_program(this: &Device, source: &str) -> Result<Program, Error> {
    unsafe {
        let mut strings = source.as_ptr() as *const i8;
        let lengths = source.len();
        let program = ffi::clCreateProgramWithSource(this.data as cl_context,
                                                     1,
                                                     &mut strings,
                                                     &lengths,
                                                     ptr::null_mut());
        if program == ptr::null_mut() {
            return Err(Error::Failed)
        }

        let mut device_id: cl_device_id = ptr::null_mut();
        if ffi::clGetContextInfo(this.data as cl_context,
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
                               ptr::null_mut()) == CL_SUCCESS {
            Ok(Program {
                data: program as usize,
                functions: &PROGRAM_FUNCTIONS,
            })
        } else {
            Err(Error::Failed)
        }
    }
}

fn create_buffer<'a>(this: &Device, protection: Protection, mut data: BufferData<'a>)
                     -> Result<Buffer<'a>, Error> {
    unsafe {
        let mut mem_flags = protection_to_mem_flags(protection);
        let (size, host_ptr);
        match data {
            BufferData::HostAllocated(ref mut buffer) => {
                if protection == Protection::ReadOnly {
                    mem_flags |= CL_MEM_USE_HOST_PTR
                } else {
                    mem_flags |= CL_MEM_COPY_HOST_PTR
                }

                size = buffer.size();
                host_ptr = buffer.as_ptr()
            }
            BufferData::Uninitialized(in_size) => {
                size = in_size;
                host_ptr = ptr::null_mut()
            }
        }

        let buffer = ffi::clCreateBuffer(this.data as cl_context,
                                         mem_flags,
                                         size,
                                         host_ptr as *mut c_void,
                                         ptr::null_mut());
        if !buffer.is_null() {
            Ok(Buffer {
                data: buffer as usize,
                functions: &BUFFER_FUNCTIONS,
                phantom: PhantomData,
            })
        } else {
            Err(Error::Failed)
        }
    }
}

// TODO(pcwalton): Support more image formats than R8.
#[cfg(target_os = "macos")]
fn create_texture(this: &Device, protection: Protection, size: &Size2D<u32>)
                  -> Result<Texture, Error> {
    unsafe {
        let properties = CFDictionary::from_CFType_pairs(&[
            (CFString::wrap_under_get_rule(kIOSurfaceWidth),
             CFNumber::from_i64(size.width as i64)),
            (CFString::wrap_under_get_rule(kIOSurfaceHeight),
             CFNumber::from_i64(size.height as i64)),
            (CFString::wrap_under_get_rule(kIOSurfaceBytesPerElement), CFNumber::from_i32(1)),
        ]);
        let surface = io_surface::new(&properties);

        let protection = protection_to_mem_flags(protection);

        let image_format = cl_image_format {
            image_channel_order: CL_R,
            image_channel_data_type: CL_UNSIGNED_INT8,
        };

        let mut error = CL_SUCCESS;

        let image = ffi::clCreateImageFromIOSurface2DAPPLE(this.data as cl_context,
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

        Ok(Texture {
            data: [image as usize, surface_ref as usize],
            functions: &TEXTURE_FUNCTIONS,
        })
    }
}

fn protection_to_mem_flags(protection: Protection) -> cl_mem_flags {
    match protection {
        Protection::ReadOnly => CL_MEM_READ_ONLY,
        Protection::WriteOnly => CL_MEM_WRITE_ONLY,
        Protection::ReadWrite => CL_MEM_READ_WRITE,
    }
}

