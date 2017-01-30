// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use error::Error;
use gl::types::GLuint;
use gl;
use profile_event::{ProfileEvent, ProfileEventFunctions};

pub static PROFILE_EVENT_FUNCTIONS: ProfileEventFunctions = ProfileEventFunctions {
    destroy: destroy,
    time_elapsed: time_elapsed,
};

unsafe fn destroy(event: &ProfileEvent) {
    let mut data = event.data as GLuint;
    gl::DeleteQueries(1, &mut data);
}

fn time_elapsed(event: &ProfileEvent) -> Result<u64, Error> {
    unsafe {
        let mut result = 0;
        gl::GetQueryObjectui64v(event.data as GLuint, gl::QUERY_RESULT, &mut result);
        Ok(result)
    }
}

