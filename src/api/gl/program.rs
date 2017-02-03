// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use gl::types::GLuint;
use gl;
use program::{Program, ProgramFunctions};

pub static PROGRAM_FUNCTIONS: ProgramFunctions = ProgramFunctions {
    destroy: destroy,
};

unsafe fn destroy(this: &Program) {
    let mut shader = 0;
    gl::GetAttachedShaders(this.data() as GLuint, 1, &mut 0, &mut shader);
    gl::UseProgram(0);
    gl::DeleteProgram(this.data() as GLuint);
    gl::DeleteShader(shader);
}

