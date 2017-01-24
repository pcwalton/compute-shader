// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use buffer::{Buffer, BufferData, Protection};
use error::Error;
use euclid::Size2D;
use program::Program;
use queue::Queue;
use texture::{Format, Texture};

pub struct Device {
    pub data: usize,
    pub functions: &'static DeviceFunctions,
}

pub struct DeviceFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Device),
    pub create_queue: extern "Rust" fn(this: &Device) -> Result<Queue, Error>,
    pub create_program: extern "Rust" fn(this: &Device, source: &str) -> Result<Program, Error>,
    pub create_buffer: extern "Rust" fn(this: &Device, protection: Protection, data: BufferData)
                                        -> Result<Buffer, Error>,
    pub create_texture: extern "Rust" fn(this: &Device,
                                         format: Format,
                                         protection: Protection,
                                         size: &Size2D<u32>)
                                         -> Result<Texture, Error>,
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

impl Device {
    #[inline]
    pub fn create_queue(&self) -> Result<Queue, Error> {
        (self.functions.create_queue)(self)
    }

    #[inline]
    pub fn create_program(&self, source: &str) -> Result<Program, Error> {
        (self.functions.create_program)(self, source)
    }

    #[inline]
    pub fn create_buffer(&self, protection: Protection, data: BufferData)
                         -> Result<Buffer, Error> {
        (self.functions.create_buffer)(self, protection, data)
    }

    #[inline]
    pub fn create_texture(&self, format: Format, protection: Protection, size: &Size2D<u32>)
                          -> Result<Texture, Error> {
        (self.functions.create_texture)(self, format, protection, size)
    }
}

