// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use buffer::Buffer;
use error::Error;
use event::Event;
use program::Program;
use texture::{Color, Texture};

pub struct Queue {
    pub data: usize,
    pub functions: &'static QueueFunctions,
}

pub struct QueueFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Queue),
    pub finish: extern "Rust" fn(this: &Queue) -> Result<(), Error>,
    pub submit_compute: extern "Rust" fn(this: &Queue,
                                         program: &Program,
                                         num_groups: &[u32],
                                         uniforms: &[(u32, Uniform)],
                                         events: &[Event])
                                         -> Result<Event, Error>,
    pub submit_clear: extern "Rust" fn(this: &Queue,
                                       texture: &Texture,
                                       color: &Color,
                                       events: &[Event])
                                       -> Result<Event, Error>,
    pub submit_read_buffer: extern "Rust" fn(this: &Queue,
                                             dest: &mut [u8],
                                             buffer: &Buffer,
                                             start: usize,
                                             events: &[Event])
                                             -> Result<Event, Error>,
}

pub enum Uniform<'a> {
    Buffer(&'a Buffer<'a>),
    Texture(&'a Texture),
    U32(u32),
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

