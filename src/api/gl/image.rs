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
use image::{ExternalImage, Format, Image, ImageFunctions};

pub static IMAGE_FUNCTIONS: ImageFunctions = ImageFunctions {
    destroy: destroy,
    bind_to: bind_to,
    width: width,
    height: height,
    format: format,
};

unsafe fn destroy(this: &Image) {
    let mut texture = this.data()[0] as GLuint;
    gl::DeleteTextures(1, &mut texture);
}

fn bind_to(this: &Image, external_texture: &ExternalImage) -> Result<(), Error> {
    unsafe {
        match *external_texture {
            ExternalImage::GlTexture(texture) => {
                let mut format = 0;
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_RECTANGLE, this.data()[0] as GLuint);
                gl::GetTexLevelParameteriv(gl::TEXTURE_RECTANGLE,
                                           0,
                                           gl::TEXTURE_INTERNAL_FORMAT,
                                           &mut format);

                gl::TextureView(texture,
                                gl::TEXTURE_RECTANGLE,
                                this.data()[0] as GLuint,
                                format as GLuint,
                                0,
                                1,
                                0,
                                1)
            }
        }
        Ok(())
    }
}

fn width(this: &Image) -> Result<u32, Error> {
    unsafe {
        let mut width = 0;
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_RECTANGLE, this.data()[0] as GLuint);
        gl::GetTexLevelParameteriv(gl::TEXTURE_RECTANGLE, 0, gl::TEXTURE_WIDTH, &mut width);
        Ok(width as u32)
    }
}

fn height(this: &Image) -> Result<u32, Error> {
    unsafe {
        let mut height = 0;
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_RECTANGLE, this.data()[0] as GLuint);
        gl::GetTexLevelParameteriv(gl::TEXTURE_RECTANGLE, 0, gl::TEXTURE_HEIGHT, &mut height);
        Ok(height as u32)
    }
}

fn format(this: &Image) -> Result<Format, Error> {
    unsafe {
        let mut internal_format = 0;
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_RECTANGLE, this.data()[0] as GLuint);
        gl::GetTexLevelParameteriv(gl::TEXTURE_RECTANGLE,
                                   0,
                                   gl::TEXTURE_INTERNAL_FORMAT,
                                   &mut internal_format);

        // This must match the definition of `Format::gl_internal_format()`.
        match internal_format as GLuint {
            gl::R8 => Ok(Format::R8),
            gl::RGBA8 => Ok(Format::RGBA8),
            gl::R32F => Ok(Format::R32F),
            _ => Err(Error::Failed),
        }
    }
}

