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
    create_device: create_device,
};

pub fn create() -> Result<Instance, Error> {
    Ok(Instance {
        data: 0,
        functions: &INSTANCE_FUNCTIONS,
    })
}

unsafe fn destroy(_: &Instance) {}

fn shading_language(_: &Instance) -> ShadingLanguage {
    ShadingLanguage::Glsl
}

fn create_device(_: &Instance) -> Result<Device, Error> {
    Ok(Device {
        data: 0,
        functions: &DEVICE_FUNCTIONS,
    })
}

