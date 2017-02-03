// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Errors.

/// The universal error type.
#[derive(Debug)]
pub enum Error {
    /// A miscellaneous error occurred.
    Failed,
    /// Compilation of the shader failed.
    ///
    /// The string represents the error message that the driver reported.
    CompileFailed(String),
    /// Shader linking failed.
    ///
    /// The string represents the error message that the driver reported.
    LinkFailed(String),
}

