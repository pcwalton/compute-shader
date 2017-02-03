// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::mem;
use std::slice;

pub struct Buffer {
    pub data: usize,
    pub functions: &'static BufferFunctions,
}

pub struct BufferFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Buffer),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Protection {
    ReadOnly = 0,
    WriteOnly = 1,
    ReadWrite = 2,
}

pub enum BufferData<'a> {
    Uninitialized(usize),
    HostAllocated(HostAllocatedData<'a>),
}

#[derive(Clone, Copy)]
pub struct HostAllocatedData<'a>(&'a [u8]);

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

impl<'a> HostAllocatedData<'a> {
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let slice = self.0;
        slice.as_ptr()
    }

    #[inline]
    pub fn size(&self) -> usize {
        let slice = self.0;
        slice.len()
    }

    #[inline]
    pub fn new<'b, T>(slice: &'b [T]) -> HostAllocatedData<'b> {
        unsafe {
            HostAllocatedData(slice::from_raw_parts(slice.as_ptr() as *const u8,
                                                    slice.len() * mem::size_of::<T>()))
        }
    }
}

