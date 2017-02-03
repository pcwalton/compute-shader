// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Objects that can be used to query how long GPU operations took.

use error::Error;

/// An object that can be used to query how long GPU operations took.
pub struct ProfileEvent {
    data: usize,
    functions: &'static ProfileEventFunctions,
}

#[doc(hidden)]
pub struct ProfileEventFunctions {
    pub destroy: unsafe extern "Rust" fn(this: &ProfileEvent),
    pub time_elapsed: extern "Rust" fn(this: &ProfileEvent) -> Result<u64, Error>,
}

impl Drop for ProfileEvent {
    fn drop(&mut self) {
        unsafe {
            (self.functions.destroy)(self)
        }
    }
}

impl ProfileEvent {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_raw_data(data: usize, functions: &'static ProfileEventFunctions)
                                -> ProfileEvent {
        ProfileEvent {
            data: data,
            functions: functions,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn data(&self) -> usize {
        self.data
    }

    /// Returns the time that this operation took in nanoseconds.
    ///
    /// If the operation has not yet completed, this function blocks until it completes.
    #[inline]
    pub fn time_elapsed(&self) -> Result<u64, Error> {
        (self.functions.time_elapsed)(self)
    }
}

