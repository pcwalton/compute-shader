// Copyright 2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use api::cl::ffi::{self, CL_FLOAT, CL_IMAGE_FORMAT, CL_IMAGE_HEIGHT, CL_IMAGE_WIDTH, CL_R};
use api::cl::ffi::{CL_RGBA, CL_SUCCESS, CL_UNORM_INT8, cl_image_format, cl_mem};
use error::Error;
use gl;
use image::{ExternalImage, Format, Image, ImageFunctions};
use std::mem;
use std::os::raw::c_void;
use std::ptr;

#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use io_surface::{IOSurface, IOSurfaceRef};

pub static IMAGE_FUNCTIONS: ImageFunctions = ImageFunctions {
    destroy: destroy,
    bind_to: bind_to,
    width: width,
    height: height,
};

#[cfg(target_os = "macos")]
unsafe fn destroy(this: &Image) {
    // Release the `IOSurfaceRef` by wrapping it with no reference count change and letting that
    // wrapper drop.
    let io_surface = mem::transmute::<usize, IOSurfaceRef>(this.data()[1]);
    IOSurface::wrap_under_create_rule(io_surface);

    ffi::clReleaseMemObject(this.data()[0] as cl_mem);
}

#[cfg(target_os = "macos")]
fn bind_to(this: &Image, external_image: &ExternalImage) -> Result<(), Error> {
    unsafe {
        match *external_image {
            ExternalImage::GlTexture(texture) => {
                let (width, height) = (try!(width(this)), try!(height(this)));

                let mut image_format = cl_image_format {
                    image_channel_order: 0,
                    image_channel_data_type: 0,
                };
                if ffi::clGetImageInfo(this.data()[0] as cl_mem,
                                       CL_IMAGE_FORMAT,
                                       mem::size_of::<cl_image_format>(),
                                       &mut image_format as *mut cl_image_format as *mut c_void,
                                       ptr::null_mut()) != CL_SUCCESS {
                    return Err(Error::Failed)
                }

                let format = match (image_format.image_channel_order,
                                    image_format.image_channel_data_type) {
                    (CL_R, CL_UNORM_INT8) => Format::R8,
                    (CL_R, CL_FLOAT) => Format::R32F,
                    (CL_RGBA, CL_UNORM_INT8) => Format::RGBA8,
                    _ => unreachable!(),
                };

                // FIXME(pcwalton): Fail more gracefully than panicking! (Really an `io-surface-rs`
                // bug.)
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_RECTANGLE, texture);
                let io_surface = mem::transmute::<usize, IOSurfaceRef>(this.data()[1]);
                let io_surface = IOSurface::wrap_under_get_rule(io_surface);
                io_surface.bind_to_gl_texture(width as i32,
                                              height as i32,
                                              format.gl_internal_format(),
                                              format.gl_format(),
                                              format.gl_type());
                Ok(())
            }
        }
    }
}

fn width(this: &Image) -> Result<u32, Error> {
    unsafe {
        let mut width = 0usize;
        if ffi::clGetImageInfo(this.data()[0] as cl_mem,
                               CL_IMAGE_WIDTH,
                               mem::size_of::<usize>(),
                               &mut width as *mut usize as *mut c_void,
                               ptr::null_mut()) == CL_SUCCESS {
            Ok(width as u32)
        } else {
            Err(Error::Failed)
        }
    }
}

fn height(this: &Image) -> Result<u32, Error> {
    unsafe {
        let mut height = 0usize;
        if ffi::clGetImageInfo(this.data()[0] as cl_mem,
                               CL_IMAGE_HEIGHT,
                               mem::size_of::<usize>(),
                               &mut height as *mut usize as *mut c_void,
                               ptr::null_mut()) == CL_SUCCESS {
            Ok(height as u32)
        } else {
            Err(Error::Failed)
        }
    }
}

