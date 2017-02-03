// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Events (a.k.a. fences) that can be waited on.

use error::Error;

/// An event (a.k.a. fence) that can be waited on.
pub struct SyncEvent {
    data: usize,
    functions: &'static SyncEventFunctions,
}

#[doc(hidden)]
pub struct SyncEventFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &SyncEvent),
    pub wait: extern "Rust" fn(this: &SyncEvent) -> Result<(), Error>,
}

impl Drop for SyncEvent {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

impl SyncEvent {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_raw_data(data: usize, functions: &'static SyncEventFunctions) -> SyncEvent {
        SyncEvent {
            data: data,
            functions: functions,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn data(&self) -> usize {
        self.data
    }

    /// Blocks the CPU until this event has occurred.
    #[inline]
    pub fn wait(&self) -> Result<(), Error> {
        (self.functions.wait)(self)
    }
}

