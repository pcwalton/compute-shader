// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::cl::ffi::{self, CL_PROFILING_COMMAND_END, CL_PROFILING_COMMAND_START, CL_SUCCESS};
use api::cl::ffi::{cl_event, cl_ulong};
use error::Error;
use profile_event::{ProfileEvent, ProfileEventFunctions};
use std::mem;
use std::os::raw::c_void;
use std::ptr;

pub static PROFILE_EVENT_FUNCTIONS: ProfileEventFunctions = ProfileEventFunctions {
    destroy: destroy,
    time_elapsed: time_elapsed,
};

unsafe fn destroy(this: &ProfileEvent) {
    ffi::clReleaseEvent(this.data as cl_event);
}

fn time_elapsed(this: &ProfileEvent) -> Result<u64, Error> {
    unsafe {
        if ffi::clWaitForEvents(1, (&this.data) as *const usize as *const cl_event) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        let mut start_time = 0;
        if ffi::clGetEventProfilingInfo(this.data as cl_event,
                                        CL_PROFILING_COMMAND_START,
                                        mem::size_of::<cl_ulong>(),
                                        &mut start_time as *mut u64 as *mut c_void,
                                        ptr::null_mut()) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        let mut end_time = 0;
        if ffi::clGetEventProfilingInfo(this.data as cl_event,
                                        CL_PROFILING_COMMAND_END,
                                        mem::size_of::<cl_ulong>(),
                                        &mut end_time as *mut u64 as *mut c_void,
                                        ptr::null_mut()) != CL_SUCCESS {
            return Err(Error::Failed)
        }

        Ok(end_time - start_time)
    }
}

