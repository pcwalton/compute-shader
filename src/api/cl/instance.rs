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
use api::cl::ffi::{self, CL_DEVICE_TYPE_GPU, CL_SUCCESS};
use device::Device;
use error::Error;
use instance::{Instance, InstanceFunctions, ShadingLanguage};
use std::ptr;

pub static INSTANCE_FUNCTIONS: InstanceFunctions = InstanceFunctions {
    destroy: destroy,
    shading_language: shading_language,
    create_device: create_device,
};

pub fn create() -> Instance {
    Instance {
        data: 0,
        functions: &INSTANCE_FUNCTIONS,
    }
}

unsafe fn destroy(_: &Instance) {}

fn shading_language(_: &Instance) -> ShadingLanguage {
    ShadingLanguage::Cl
}

fn create_device(_: &Instance) -> Result<Device, Error> {
    unsafe {
        let (mut device_id, mut num_devices) = (ptr::null_mut(), 0);
        if ffi::clGetDeviceIDs(ptr::null_mut(),
                               CL_DEVICE_TYPE_GPU,
                               1,
                               &mut device_id,
                               &mut num_devices) != CL_SUCCESS || num_devices == 0 {
            return Err(Error)
        }

        let context = ffi::clCreateContext(ptr::null_mut(),
                                           1,
                                           &device_id,
                                           None,
                                           ptr::null_mut(),
                                           ptr::null_mut());
        if context.is_null() {
            return Err(Error)
        }

        Ok(Device {
            data: context as usize,
            functions: &DEVICE_FUNCTIONS, 
        })
    }
}

