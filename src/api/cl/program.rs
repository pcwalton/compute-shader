// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::cl::ffi::{self, CL_KERNEL_PROGRAM, cl_kernel, cl_program};
use program::{Program, ProgramFunctions};
use std::mem;
use std::os::raw::c_void;
use std::ptr;

pub static PROGRAM_FUNCTIONS: ProgramFunctions = ProgramFunctions {
    destroy: destroy,
};

unsafe fn destroy(this: &Program) {
    let mut program = ptr::null_mut();
    ffi::clGetKernelInfo(this.data as cl_kernel,
                         CL_KERNEL_PROGRAM,
                         mem::size_of::<cl_program>(),
                         &mut program as *mut cl_program as *mut c_void,
                         ptr::null_mut());

    ffi::clReleaseKernel(this.data as cl_kernel);
    ffi::clReleaseProgram(program);
}

