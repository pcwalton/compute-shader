// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Images, which are essentially read-write textures.

use error::Error;
use gl::types::GLuint;
use gl;

/// An image, which is essentially a read-write texture.
pub struct Image {
    data: [usize; 2],
    functions: &'static ImageFunctions,
}

#[doc(hidden)]
pub struct ImageFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Image),
    pub bind_to: extern "Rust" fn(this: &Image, external_image: &ExternalImage)
                                  -> Result<(), Error>,
    pub width: extern "Rust" fn(this: &Image) -> Result<u32, Error>,
    pub height: extern "Rust" fn(this: &Image) -> Result<u32, Error>,
}

/// An external resource that can be made to refer to this image.
pub enum ExternalImage {
    /// The name of an OpenGL texture.
    GlTexture(u32),
}

/// The image format of a texture.
///
/// TODO(pcwalton): Support more.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Format {
    /// 8-bit unsigned single-channel.
    R8,
    /// 32-bit single-channel floating-point.
    R32F,
}

/// A color.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    /// A 4-channel color.
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
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_raw_data(data: [usize; 2], functions: &'static ImageFunctions) -> Image {
        Image {
            data: data,
            functions: functions,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn data(&self) -> [usize; 2] {
        self.data
    }

    /// Makes `external_image` reflect the contents of this image.
    ///
    /// This is useful in order to render an image created using a compute shader with OpenGL, for
    /// example.
    #[inline]
    pub fn bind_to(&self, external_image: &ExternalImage) -> Result<(), Error> {
        (self.functions.bind_to)(self, external_image)
    }

    /// Returns the width of this image in pixels.
    #[inline]
    pub fn width(&self) -> Result<u32, Error> {
        (self.functions.width)(self)
    }

    /// Returns the height of this image in pixels.
    #[inline]
    pub fn height(&self) -> Result<u32, Error> {
        (self.functions.height)(self)
    }
}

impl Format {
    /// Returns the value that should be passed as the `format` parameter to `glTexImage2D()` to
    /// create a texture matching this image format.
    #[inline]
    pub fn gl_format(self) -> GLuint {
        match self {
            Format::R8 | Format::R32F => gl::RED,
        }
    }

    /// Returns the value that should be passed as the `type` parameter to `glTexImage2D()` to
    /// create a texture matching this image format.
    #[inline]
    pub fn gl_type(self) -> GLuint {
        match self {
            Format::R8 => gl::UNSIGNED_BYTE,
            Format::R32F => gl::FLOAT,
        }
    }

    /// Returns the OpenGL image format corresponding to this image format.
    #[inline]
    pub fn gl_internal_format(self) -> GLuint {
        match self {
            Format::R8 => gl::R8,
            Format::R32F => gl::R32F,
        }
    }
}

