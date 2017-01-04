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
use texture::{ExternalTexture, Texture, TextureFunctions};

pub static TEXTURE_FUNCTIONS: TextureFunctions = TextureFunctions {
    destroy: destroy,
    bind_to: bind_to,
};

unsafe fn destroy(this: &Texture) {
    let mut texture = this.data[0] as GLuint;
    gl::DeleteTextures(1, &mut texture);
}

fn bind_to(this: &Texture, external_texture: &ExternalTexture) -> Result<(), Error> {
    unsafe {
        match *external_texture {
            ExternalTexture::Gl(texture) => {
                gl::TextureView(texture,
                                gl::TEXTURE_2D,
                                this.data[0] as GLuint,
                                gl::R8,
                                0,
                                1,
                                0,
                                1)
            }
        }
        Ok(())
    }
}

