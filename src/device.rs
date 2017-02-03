// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A GPU that supports compute.

use buffer::{Buffer, BufferData, Protection};
use error::Error;
use euclid::Size2D;
use image::{Format, Image};
use program::Program;
use queue::Queue;

/// A GPU that supports compute.
pub struct Device {
    data: usize,
    functions: &'static DeviceFunctions,
}

#[doc(hidden)]
pub struct DeviceFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Device),
    pub create_queue: extern "Rust" fn(this: &Device) -> Result<Queue, Error>,
    pub create_program: extern "Rust" fn(this: &Device, source: &str) -> Result<Program, Error>,
    pub create_buffer: extern "Rust" fn(this: &Device, protection: Protection, data: BufferData)
                                        -> Result<Buffer, Error>,
    pub create_image: extern "Rust" fn(this: &Device,
                                       format: Format,
                                       protection: Protection,
                                       size: &Size2D<u32>)
                                       -> Result<Image, Error>,
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

impl Device {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_raw_data(data: usize, functions: &'static DeviceFunctions) -> Device {
        Device {
            data: data,
            functions: functions,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn data(&self) -> usize {
        self.data
    }

    /// Creates a new command queue on which jobs can be submitted.
    #[inline]
    pub fn create_queue(&self) -> Result<Queue, Error> {
        (self.functions.create_queue)(self)
    }

    /// Creates, compiles, and links a new compute program to execute on the GPU with the given
    /// source.
    ///
    /// The supplied source must conform to the result of `Instance::shading_language()`.
    #[inline]
    pub fn create_program(&self, source: &str) -> Result<Program, Error> {
        (self.functions.create_program)(self, source)
    }

    /// Creates a new block of GPU memory with the given GPU-side protection, initialized with the
    /// supplied data.
    #[inline]
    pub fn create_buffer(&self, protection: Protection, data: BufferData)
                         -> Result<Buffer, Error> {
        (self.functions.create_buffer)(self, protection, data)
    }

    /// Creates a new image of the given format, GPU-side protection, and size.
    ///
    /// The initial contents of the image are undefined.
    #[inline]
    pub fn create_image(&self, format: Format, protection: Protection, size: &Size2D<u32>)
                        -> Result<Image, Error> {
        (self.functions.create_image)(self, format, protection, size)
    }
}

