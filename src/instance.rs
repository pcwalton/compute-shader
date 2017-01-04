// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use device::Device;
use error::Error;

pub struct Instance {
    pub data: usize,
    pub functions: &'static InstanceFunctions,
}

pub struct InstanceFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Instance),
    pub shading_language: extern "Rust" fn(this: &Instance) -> ShadingLanguage,
    pub create_device: extern "Rust" fn(this: &Instance) -> Result<Device, Error>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShadingLanguage {
    Cl,
    Glsl,
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

