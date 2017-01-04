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

pub struct Texture {
    pub data: [usize; 2],
    pub functions: &'static TextureFunctions,
}

pub struct TextureFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Texture),
    pub bind_to: extern "Rust" fn(this: &Texture, external_texture: &ExternalTexture)
                                  -> Result<(), Error>,
}

pub enum ExternalTexture {
    Gl(u32),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    UInt(u32, u32, u32, u32),
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

