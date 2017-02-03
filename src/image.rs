// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use error::Error;
use gl::types::GLuint;
use gl;

pub struct Image {
    pub data: [usize; 2],
    pub functions: &'static ImageFunctions,
}

pub struct ImageFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Image),
    pub bind_to: extern "Rust" fn(this: &Image, external_image: &ExternalImage) -> Result<(), Error>,
    pub width: extern "Rust" fn(this: &Image) -> Result<u32, Error>,
    pub height: extern "Rust" fn(this: &Image) -> Result<u32, Error>,
}

pub enum ExternalImage {
    GlTexture(u32),
}

/// The format of a texture.
///
/// TODO(pcwalton): Support more.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Format {
    /// 8-bit unsigned single-channel.
    R8,
    /// 32-bit single-channel floating-point.
    R32F,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    UInt(u32, u32, u32, u32),
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

impl Image {
    #[inline]
    pub fn bind_to(&self, external_image: &ExternalImage) -> Result<(), Error> {
        (self.functions.bind_to)(self, external_image)
    }

    #[inline]
    pub fn width(&self) -> Result<u32, Error> {
        (self.functions.width)(self)
    }

    #[inline]
    pub fn height(&self) -> Result<u32, Error> {
        (self.functions.height)(self)
    }
}

impl Format {
    #[inline]
    pub fn gl_format(self) -> GLuint {
        match self {
            Format::R8 | Format::R32F => gl::RED,
        }
    }

    #[inline]
    pub fn gl_type(self) -> GLuint {
        match self {
            Format::R8 => gl::UNSIGNED_BYTE,
            Format::R32F => gl::FLOAT,
        }
    }

    #[inline]
    pub fn gl_internal_format(self) -> GLuint {
        match self {
            Format::R8 => gl::R8,
            Format::R32F => gl::R32F,
        }
    }
}
