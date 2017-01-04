// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::marker::PhantomData;

pub struct Buffer<'a> {
    pub data: usize,
    pub functions: &'static BufferFunctions,
    pub phantom: PhantomData<&'a ()>,
}

pub struct BufferFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &Buffer),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Protection {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub enum BufferData<'a> {
    Uninitialized(usize),
    HostAllocated(&'a mut [u8]),
}

impl<'a> Drop for Buffer<'a> {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

