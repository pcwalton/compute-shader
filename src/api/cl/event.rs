// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::cl::ffi::{self, CL_SUCCESS, cl_event};
use error::Error;
use event::{Event, EventFunctions};

pub static EVENT_FUNCTIONS: EventFunctions = EventFunctions {
    destroy: destroy,
    wait: wait,
};

unsafe fn destroy(this: &Event) {
    ffi::clReleaseEvent(this.data as cl_event);
}

fn wait(this: &Event) -> Result<(), Error> {
    unsafe {
        let event = this.data as cl_event;
        if ffi::clWaitForEvents(1, &event) == CL_SUCCESS {
            Ok(())
        } else {
            Err(Error::Failed)
        }
    }
}

