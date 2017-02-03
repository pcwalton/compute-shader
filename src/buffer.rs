// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Data buffers on the GPU.

use std::mem;
use std::slice;

/// A data buffer on the GPU.
pub struct Buffer {
    data: usize,
    functions: &'static BufferFunctions,
}

#[doc(hidden)]
pub struct BufferFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Buffer),
}

/// Memory protection from the GPU side. (The CPU is always free to perform whatever reads and
/// writes it wants.)
///
/// Do not rely on the driver to enforce this.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Protection {
    /// The GPU may only read from this buffer.
    ReadOnly = 0,
    /// The GPU may only write to the buffer.
    ///
    /// Note that atomic writes count as reads too, so use `ReadWrite` instead if you're using
    /// them.
    WriteOnly = 1,
    /// The GPU may read and write to the buffer.
    ReadWrite = 2,
}

/// Where the initial data for a buffer comes from.
pub enum BufferData<'a> {
    /// The data is undefined data of the given size.
    Uninitialized(usize),
    /// The data is initialized with the given contents.
    HostAllocated(HostAllocatedData<'a>),
}

/// CPU-side data to initialize a buffer store with.
#[derive(Clone, Copy)]
pub struct HostAllocatedData<'a>(&'a [u8]);

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

impl Buffer {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_raw_data(data: usize, functions: &'static BufferFunctions) -> Buffer {
        Buffer {
            data: data,
            functions: functions,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn data(&self) -> usize {
        self.data
    }
}

impl<'a> HostAllocatedData<'a> {
    /// Returns a raw pointer to the data.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let slice = self.0;
        slice.as_ptr()
    }

    /// Returns the size of the data.
    #[inline]
    pub fn size(&self) -> usize {
        let slice = self.0;
        slice.len()
    }

    /// Constructs a `HostAllocatedData` from the given slice.
    #[inline]
    pub fn new<'b, T>(slice: &'b [T]) -> HostAllocatedData<'b> {
        unsafe {
            HostAllocatedData(slice::from_raw_parts(slice.as_ptr() as *const u8,
                                                    slice.len() * mem::size_of::<T>()))
        }
    }
}

