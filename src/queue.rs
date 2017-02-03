// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Queues on which compute jobs can be submitted.

use buffer::Buffer;
use error::Error;
use image::{Color, Image};
use profile_event::ProfileEvent;
use program::Program;
use sync_event::SyncEvent;

/// A queue on which compute jobs can be submitted.
pub struct Queue {
    data: usize,
    functions: &'static QueueFunctions,
}

#[doc(hidden)]
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
                                       image: &Image,
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

/// An argument to a program.
pub enum Uniform<'a> {
    /// A reference to a GPU-side memory buffer.
    Buffer(&'a Buffer),
    /// A reference to an image on the GPU.
    Image(&'a Image),
    /// A 32-bit unsigned integer value.
    U32(u32),
    /// A vector of 4 32-bit unsigned integers.
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
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_raw_data(data: usize, functions: &'static QueueFunctions) -> Queue {
        Queue {
            data: data,
            functions: functions,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn data(&self) -> usize {
        self.data
    }

    /// Submits all queued commands to the GPU.
    ///
    /// This does *not* wait for the commands to finish. It does, however, guarantee that they will
    /// complete (or fail) in finite time.
    #[inline]
    pub fn flush(&self) -> Result<(), Error> {
        (self.functions.flush)(self)
    }

    /// Submits commands to the GPU and waits for them to complete.
    #[inline]
    pub fn finish(&self) -> Result<(), Error> {
        (self.functions.finish)(self)
    }

    /// Instructs the GPU to execute the given program.
    ///
    /// * `program` specifies the program.
    ///
    /// * `num_groups` specifies the number of workgroups in each dimension. It must be an array of
    ///   between 1 and 3 nonzero values.
    ///
    /// * `uniforms` specifies the values of arguments used to invoke the program. The first
    ///   element in the tuple is the argument index; the second is the actual value of the
    ///   argument (see `Uniform`).
    ///
    /// * `events` is a list of sync events that must complete before execution of this program can
    ///   begin.
    ///
    /// Returns a profiling event that can be used to time the execution of this program.
    #[inline]
    pub fn submit_compute(&self,
                          program: &Program,
                          num_groups: &[u32],
                          uniforms: &[(u32, Uniform)],
                          events: &[SyncEvent])
                          -> Result<ProfileEvent, Error> {
        (self.functions.submit_compute)(self, program, num_groups, uniforms, events)
    }

    /// Instructs the GPU to clear the given image to a solid color.
    ///
    /// * `image` specifies the image to clear.
    ///
    /// * `color` specifies the color to fill with.
    ///
    /// * `events` is a list of sync events that must complete before this operation can begin.
    ///
    /// Returns a profiling event that can be used to query how long it took to clear the image.
    #[inline]
    pub fn submit_clear(&self, image: &Image, color: &Color, events: &[SyncEvent])
                        -> Result<ProfileEvent, Error> {
        (self.functions.submit_clear)(self, image, color, events)
    }

    /// Reads a buffer from the GPU to main memory.
    ///
    /// * `dest` specifies the buffer to read to.
    ///
    /// * `buffer` specifies the buffer to read from.
    ///
    /// * `start` specifies the position in the buffer to begin reading from.
    ///
    /// * `events` is a list of sync events that must complete before this operation can begin.
    ///
    /// Returns a profiling event that can be used to query how long this operation took.
    ///
    /// This operation blocks until completion.
    #[inline]
    pub fn submit_read_buffer(&self,
                              dest: &mut [u8],
                              buffer: &Buffer,
                              start: usize,
                              events: &[SyncEvent])
                              -> Result<ProfileEvent, Error> {
        (self.functions.submit_read_buffer)(self, dest, buffer, start, events)
    }

    /// Returns a sync event that can be used to wait until the GPU has finished executing all
    /// commands up to the point at which this is called.
    #[inline]
    pub fn submit_sync_event(&self) -> Result<SyncEvent, Error> {
        (self.functions.submit_sync_event)(self)
    }
}

