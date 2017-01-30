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
use profile_event::ProfileEvent;
use program::Program;
use sync_event::SyncEvent;
use texture::{Color, Texture};

pub struct Queue {
    pub data: usize,
    pub functions: &'static QueueFunctions,
}

pub struct QueueFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Queue),
    pub flush: extern "Rust" fn(this: &Queue) -> Result<(), Error>,
    pub finish: extern "Rust" fn(this: &Queue) -> Result<(), Error>,
    pub submit_compute: extern "Rust" fn(this: &Queue,
                                         program: &Program,
                                         num_groups: &[u32],
                                         uniforms: &[(u32, Uniform)],
                                         events: &[SyncEvent])
                                         -> Result<ProfileEvent, Error>,
    pub submit_clear: extern "Rust" fn(this: &Queue,
                                       texture: &Texture,
                                       color: &Color,
                                       events: &[SyncEvent])
                                       -> Result<ProfileEvent, Error>,
    pub submit_read_buffer: extern "Rust" fn(this: &Queue,
                                             dest: &mut [u8],
                                             buffer: &Buffer,
                                             start: usize,
                                             events: &[SyncEvent])
                                             -> Result<ProfileEvent, Error>,
    pub submit_sync_event: extern "Rust" fn(this: &Queue) -> Result<SyncEvent, Error>,
}

pub enum Uniform<'a> {
    Buffer(&'a Buffer),
    Texture(&'a Texture),
    U32(u32),
    UVec4([u32; 4]),
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

impl Queue {
    #[inline]
    pub fn flush(&self) -> Result<(), Error> {
        (self.functions.flush)(self)
    }

    #[inline]
    pub fn finish(&self) -> Result<(), Error> {
        (self.functions.finish)(self)
    }

    #[inline]
    pub fn submit_compute(&self,
                          program: &Program,
                          num_groups: &[u32],
                          uniforms: &[(u32, Uniform)],
                          events: &[SyncEvent])
                          -> Result<ProfileEvent, Error> {
        (self.functions.submit_compute)(self, program, num_groups, uniforms, events)
    }

    #[inline]
    pub fn submit_clear(&self, texture: &Texture, color: &Color, events: &[SyncEvent])
                        -> Result<ProfileEvent, Error> {
        (self.functions.submit_clear)(self, texture, color, events)
    }

    #[inline]
    pub fn submit_read_buffer(&self,
                              dest: &mut [u8],
                              buffer: &Buffer,
                              start: usize,
                              events: &[SyncEvent])
                              -> Result<ProfileEvent, Error> {
        (self.functions.submit_read_buffer)(self, dest, buffer, start, events)
    }

    #[inline]
    pub fn submit_sync_event(&self) -> Result<SyncEvent, Error> {
        (self.functions.submit_sync_event)(self)
    }
}

