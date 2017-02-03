// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::gl::device::DEVICE_FUNCTIONS;
use device::Device;
use error::Error;
use instance::{Instance, InstanceFunctions, ShadingLanguage};

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
    ShadingLanguage::Glsl
}

fn open_device(_: &Instance) -> Result<Device, Error> {
    unsafe {
        Ok(Device::from_raw_data(0, &DEVICE_FUNCTIONS))
    }
}

