// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use buffer::{Buffer, BufferFunctions};
use gl::types::GLuint;
use gl;

pub static BUFFER_FUNCTIONS: BufferFunctions = BufferFunctions {
    destroy: destroy,
};

unsafe fn destroy(this: &Buffer) {
    let mut buffer = this.data() as GLuint;
    gl::DeleteBuffers(1, &mut buffer)
}

