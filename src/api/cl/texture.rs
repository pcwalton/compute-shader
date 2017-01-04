// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::cl::ffi::{self, CL_IMAGE_HEIGHT, CL_IMAGE_WIDTH, cl_mem};
use error::Error;
use gl;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use texture::{ExternalTexture, Texture, TextureFunctions};

#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use io_surface::{IOSurface, IOSurfaceRef};

pub static TEXTURE_FUNCTIONS: TextureFunctions = TextureFunctions {
    destroy: destroy,
    bind_to: bind_to,
};

#[cfg(target_os = "macos")]
unsafe fn destroy(this: &Texture) {
    // Release the `IOSurfaceRef` by wrapping it with no reference count change and letting that
    // wrapper drop.
    let io_surface = mem::transmute::<usize, IOSurfaceRef>(this.data[1]);
    IOSurface::wrap_under_create_rule(io_surface);

    ffi::clReleaseMemObject(this.data[0] as cl_mem);
}

#[cfg(target_os = "macos")]
fn bind_to(this: &Texture, external_texture: &ExternalTexture) -> Result<(), Error> {
    unsafe {
        match *external_texture {
            ExternalTexture::Gl(texture) => {
                let (mut width, mut height): (usize, usize) = (0, 0);
                ffi::clGetImageInfo(this.data[0] as cl_mem,
                                    CL_IMAGE_WIDTH,
                                    mem::size_of::<usize>(),
                                    &mut width as *mut usize as *mut c_void,
                                    ptr::null_mut());
                ffi::clGetImageInfo(this.data[0] as cl_mem,
                                    CL_IMAGE_HEIGHT,
                                    mem::size_of::<usize>(),
                                    &mut height as *mut usize as *mut c_void,
                                    ptr::null_mut());

                // TODO(pcwalton): Support formats other than R8!
                // FIXME(pcwalton): Fail more gracefully than panicking! (Really an `io-surface-rs`
                // bug.)
                gl::BindTexture(gl::TEXTURE_RECTANGLE, texture);
                let io_surface = mem::transmute::<usize, IOSurfaceRef>(this.data[1]);
                let io_surface = IOSurface::wrap_under_get_rule(io_surface);
                io_surface.bind_to_gl_texture(width as i32,
                                              height as i32,
                                              gl::R8,
                                              gl::RED,
                                              gl::UNSIGNED_BYTE);
                Ok(())
            }
        }
    }
}

