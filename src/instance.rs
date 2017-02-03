// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Instances of the `compute-shader` library.

use api;
use device::Device;
use error::Error;

/// An instance of the `compute-shader` library.
///
/// This wraps the underlying platform-specific API (currently OpenCL 1.2+ or OpenGL 4.3+).
pub struct Instance {
    data: usize,
    functions: &'static InstanceFunctions,
}

#[doc(hidden)]
pub struct InstanceFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Instance),
    pub shading_language: extern "Rust" fn(this: &Instance) -> ShadingLanguage,
    pub open_device: extern "Rust" fn(this: &Instance) -> Result<Device, Error>,
}

/// The shading language supported by this instance.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShadingLanguage {
    /// OpenCL 1.2+.
    Cl,
    /// OpenGL Shading Language 4.3+.
    Glsl,
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

impl Instance {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_raw_data(data: usize, functions: &'static InstanceFunctions) -> Instance {
        Instance {
            data: data,
            functions: functions,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn data(&self) -> usize {
        self.data
    }

    /// Returns the shading language accepted by `Device::create_program()`.
    #[inline]
    pub fn shading_language(&self) -> ShadingLanguage {
        (self.functions.shading_language)(self)
    }

    /// Opens a handle to the GPU.
    #[inline]
    pub fn open_device(&self) -> Result<Device, Error> {
        (self.functions.open_device)(self)
    }

    /// Initializes the library and returns a new instance.
    #[cfg(target_os = "macos")]
    pub fn new() -> Result<Instance, Error> {
        api::cl::instance::create()
    }

    /// Initializes the library and returns a new instance.
    #[cfg(not(target_os = "macos"))]
    pub fn new() -> Result<Instance, Error> {
        api::gl::instance::create()
    }
}

