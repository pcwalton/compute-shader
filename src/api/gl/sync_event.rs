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
use gl::types::{GLsync, GLuint64};
use gl;
use sync_event::{SyncEvent, SyncEventFunctions};

const TIMEOUT: GLuint64 = 1_000_000_000_000;

pub static SYNC_EVENT_FUNCTIONS: SyncEventFunctions = SyncEventFunctions {
    destroy: destroy,
    wait: wait,
};

unsafe fn destroy(event: &SyncEvent) {
    let data = event.data() as GLsync;
    gl::DeleteSync(data);
}

fn wait(event: &SyncEvent) -> Result<(), Error> {
    unsafe {
        gl::ClientWaitSync(event.data() as GLsync, gl::SYNC_FLUSH_COMMANDS_BIT, TIMEOUT);
        Ok(())
    }
}

