// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::cl::device::DEVICE_FUNCTIONS;
use api::cl::ffi::{self, CL_DEVICE_TYPE_GPU, CL_DEVICE_NAME, CL_SUCCESS, cl_device_id};
use device::Device;
use error::Error;
use gl;
use instance::{Instance, InstanceFunctions, ShadingLanguage};
use libc;
use std::cmp;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::slice;

pub static INSTANCE_FUNCTIONS: InstanceFunctions = InstanceFunctions {
    destroy: destroy,
    shading_language: shading_language,
    open_device: open_device,
};

pub fn create() -> Result<Instance, Error> {
    unsafe {
        Ok(Instance::from_raw_data(0, &INSTANCE_FUNCTIONS))
    }
}

unsafe fn destroy(_: &Instance) {}

fn shading_language(_: &Instance) -> ShadingLanguage {
    ShadingLanguage::Cl
}

fn open_device(_: &Instance) -> Result<Device, Error> {
    unsafe {
        let mut num_devices = 0;
        if ffi::clGetDeviceIDs(ptr::null_mut(),
                               CL_DEVICE_TYPE_GPU,
                               0,
                               ptr::null_mut(),
                               &mut num_devices) != CL_SUCCESS || num_devices == 0 {
            return Err(Error::Failed)
        }

        let mut device_ids: Vec<cl_device_id> = vec![ptr::null_mut(); num_devices as usize];
        if ffi::clGetDeviceIDs(ptr::null_mut(),
                               CL_DEVICE_TYPE_GPU,
                               num_devices,
                               device_ids.as_mut_ptr(),
                               ptr::null_mut()) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        // Make sure the OpenCL vendor matches the current OpenGL vendor. Otherwise, on dual-GPU
        // systems, we might end up with multiple GPUs in use!
        //
        // We choose the OpenCL device for which the name has the longest substring in common with
        // the OpenGL renderer.
        //
        // FIXME(pcwalton): This is a really hacky approach. I apologize.
        let gl_renderer = to_slice(gl::GetString(gl::RENDERER));
        let mut best_device_id = None;
        for &device_id in &device_ids {
            let mut name_len = 0;
            if ffi::clGetDeviceInfo(device_id,
                                    CL_DEVICE_NAME,
                                    0,
                                    ptr::null_mut(),
                                    &mut name_len) != CL_SUCCESS {
                return Err(Error::Failed)
            }

            let mut name: Vec<u8> = vec![0; name_len];
            if ffi::clGetDeviceInfo(device_id,
                                    CL_DEVICE_NAME,
                                    name_len,
                                    name.as_mut_ptr() as *mut c_void,
                                    ptr::null_mut()) != CL_SUCCESS {
                return Err(Error::Failed)
            }

            // Strip the trailing null.
            name.pop(); 

            let score = longest_common_substring(gl_renderer, &name);
            best_device_id = match best_device_id {
                Some((best_score, _)) if score > best_score => Some((score, device_id)),
                Some(_) => best_device_id,
                None => Some((score, device_id)),
            }
        }

        let device_id = best_device_id.unwrap().1;
        let context = ffi::clCreateContext(ptr::null_mut(),
                                           1,
                                           &device_id,
                                           None,
                                           ptr::null_mut(),
                                           ptr::null_mut());
        if context.is_null() {
            return Err(Error::Failed)
        }

        Ok(Device::from_raw_data(context as usize, &DEVICE_FUNCTIONS))
    }
}

unsafe fn to_slice<'a>(p: *const u8) -> &'a [u8] {
    slice::from_raw_parts(p, libc::strlen(p as *const c_char))
}

// Dynamic programming algorithm: https://en.wikipedia.org/wiki/Longest_common_substring_problem
fn longest_common_substring(a: &[u8], b: &[u8]) -> usize {
    let width = b.len() + 1;
    let mut longest_length = 0;
    let mut lengths = vec![0; (a.len() + 1) * width];
    for (i, &ac) in a.iter().enumerate() {
        for (j, &bc) in b.iter().enumerate() {
            if ac == bc {
                let this_length = lengths[i * width + j] + 1;
                lengths[(i + 1) * width + j + 1] = this_length;
                longest_length = cmp::max(longest_length, this_length)
            }
        }
    }
    longest_length
}

