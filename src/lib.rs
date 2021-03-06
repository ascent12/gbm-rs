// Copyright (c) 2015 Scott Anderson <ascent12@hotmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! An interface to libgbm, a generic buffer manager for Linux which is provided by Mesa.
//!
//! Libgbm can be used to retrieve framebuffers from GPUs in a driver-independant manner.

#![crate_name = "gbm_rs"]
#![crate_type = "lib"]

extern crate libc;

use std::os::unix::prelude::*;
use libc::{
    c_int,
    c_void,
    uint32_t,
    uint64_t,
    size_t,
};

/// Analogous to gbm_device.
///
/// Used to perform memory allocations for a DRM device.
pub struct Device {
    ptr: *const gbm_device
}

impl Device {
    /// Creates a Device for allocating Buffers
    ///
    /// The file descriptor passed in is used by the backend to communicate with
    /// platform for allocating the memory. For allocations using DRI this would be
    /// the file descriptor returned when opening a device such as ```/dev/dri/card0```
    ///
    /// # Arguments
    ///
    /// fd: The file descriptor for a backend specific device
    ///
    /// # Returns
    ///
    /// The newly created struct gbm_device.
    /// If the creation of the device failed None will be returned.
    ///
    /// # Example
    /// ```
    /// extern crate gbm_rs as gbm;
    ///
    /// use std::fs::OpenOptions;
    /// use std::os::unix::prelude::*;
    ///
    /// let file = OpenOptions::new().read(true).write(true).open("/dev/dri/card0").unwrap();
    ///
    /// let device = gbm::Device::from_fd(file.as_raw_fd()).unwrap();
    /// ```
    pub fn from_fd(fd: RawFd) -> Option<Device> {
        unsafe {
            let dev = gbm_create_device(fd);

            if dev.is_null() {
                return None;
            }

            return Some(Device { ptr: dev });
        }
    }

    /// Test if a format is supported for a given set of usage flags
    ///
    /// # Arguments
    ///
    /// format: The fourcc code to test
    ///
    /// usage: A bitmask of the usages to test the format against
    ///
    /// # Returns
    ///
    /// true if the format is supported otherwise false
    pub fn is_format_supported(&self, format: u32, usage: u32) -> bool {
        unsafe { gbm_device_is_format_supported(self.ptr, format, usage) != 0 }
    }

    /// Returns the file descriptor for the Device
    ///
    /// # Returns
    ///
    /// The Fd that the Device was created with
    ///
    /// # Example
    /// ```
    /// # extern crate gbm_rs as gbm;
    /// # use std::fs::OpenOptions;
    /// # use std::os::unix::prelude::*;
    /// # let file = OpenOptions::new().read(true).write(true).open("/dev/dri/card0").unwrap();
    /// # let device = gbm::Device::from_fd(file.as_raw_fd()).unwrap();
    /// let fd = device.fd();
    ///
    /// assert_eq!(fd, file.as_raw_fd());
    /// ```
    pub fn fd(&self) -> RawFd {
        unsafe { gbm_device_get_fd(self.ptr) }
    }

    /// Returns the gbm_device for the Device
    ///
    /// # Returns
    ///
    /// A pointer to the gbm_device used to create the Device.
    pub fn c_struct(&self) -> *const gbm_device {
        self.ptr
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { gbm_device_destroy(self.ptr) }
    }
}

/// Analogous to gbm_surface
///
/// Represents an area where a buffer object will be displayed.
pub struct Surface {
    ptr: *const gbm_surface
}

impl Surface {
    /// Allocate a Surface object
    ///
    /// # Arguments
    ///
    /// dev: The Device returned from Device::from_fd()
    ///
    /// width: The width for the surface
    ///
    /// height: The height for the surface
    ///
    /// format: The fourcc code for the surface
    ///
    /// flags: A bitmask of the flags for this surface
    ///
    /// # Returns
    ///
    /// A newly allocated surface.
    /// If an error occurs during allocation ```None``` will be returned.
    ///
    /// # Example
    /// ```
    /// # extern crate gbm_rs as gbm;
    /// # use std::fs::OpenOptions;
    /// # use std::os::unix::prelude::*;
    /// # let file = OpenOptions::new().read(true).write(true).open("/dev/dri/card0").unwrap();
    /// # let device = gbm::Device::from_fd(file.as_raw_fd()).unwrap();
    /// let surface = gbm::Surface::new(&device, 1920, 1080,
    ///                                 gbm::format::XRGB8888, // GBM_FORMAT_XRGB8888
    ///                                 gbm::USE_SCANOUT | gbm::USE_RENDERING).unwrap();
    /// ```
    pub fn new(dev: &Device, width: u32, height: u32,
                       format: u32, flags: u32) -> Option<Surface> {
        unsafe {
            let surf = gbm_surface_create(dev.ptr, width, height,
                                          format, flags);

            if surf.is_null() {
                return None;
            }

            return Some(Surface { ptr: surf });
        }
    }

    /// Returns whether or not a surface has free (non-locked) buffers
    ///
    /// Before starting a new frame, the surface must have a buffer
    /// available for rendering. Initially, a gbm surface will have a free
    /// buffer, but after one of more buffers have been locked,
    /// the application must check for a free buffer before rendering.
    ///
    /// If a surface doesn't have a free buffer, the application must
    /// return a buffer to the surface using ```release_buffer()```
    /// and after that, the application can query for free buffers again.
    ///
    /// # Returns
    ///
    /// ```true``` if the surface has free buffers, ```false``` otherwise
    pub fn has_free_buffers(&self) -> bool {
        unsafe { gbm_surface_has_free_buffers(self.ptr) != 0 }
    }

    /// Lock rendering to the surface's current front buffer until it is
    /// released with ```release_ buffer()```
    ///
    /// This function must be called exactly once after calling
    /// eglSwapBuffers. Calling it before any eglSwapBuffer has happend
    /// on the surface or two or more times after eglSwapBuffers is an
    /// error. A new BufferObject representing the new front buffer is returned. On
    /// multiple invocations, all the returned BufferObjects must be released in
    /// order to release the actual surface buffer.
    ///
    /// # Returns
    ///
    /// A buffer object that should be released with ```release_buffer()```
    /// when no longer needed.
    /// If an error occurs this function returns ```None```.
    ///
    /// # Example
    /// ```ignore
    /// // Render something
    ///
    /// let buffer = surface.lock_front_buffer().unwrap();
    ///
    /// // Output to the screen, etc.
    ///
    /// surface.release_buffer(buffer);
    /// ```
    pub fn lock_front_buffer(&self) -> Option<BufferObject> {
        unsafe {
            let bo = gbm_surface_lock_front_buffer(self.ptr);
            if bo.is_null() {
                return None;
            }

            return Some(BufferObject { ptr: bo, manual: false });
        }
    }

    /// Release a locked buffer obtained ```lock_front_buffer()```
    ///
    /// Returns the underlying buffer to the Surface. Releasing a BufferObject
    /// will typically ```has_free_buffer()``` return true and thus
    /// allow rendering the next frame, but not always. The implementation
    /// may choose to destroy the BufferObject immediately or reuse it, in which case
    /// the user data associated with it is unchanged.
    ///
    /// # Arguments
    ///
    /// bo: The BufferObject to be released
    pub fn release_buffer(&self, bo: BufferObject) {
        unsafe { gbm_surface_release_buffer(self.ptr, bo.ptr) }
    }

    /// Returns the gbm_surface for the Surface
    ///
    /// # Returns
    ///
    /// A pointer to the gbm_surface used to create the Surface.
    pub fn c_struct(&self) -> *const gbm_surface {
        self.ptr
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { gbm_surface_destroy(self.ptr) }
    }
}

/// Analogous to gbm_bo
pub struct BufferObject {
    ptr: *const gbm_bo,
    // To make sure we only free gbm_bo's from gbm_bo_create()
    // and NOT gbm_surface_lock_front_buffer()
    manual: bool,
}

impl BufferObject {
    /// Allocate a buffer object for the given dimensions
    ///
    /// # Arguments
    ///
    /// dev: The Device returned from Device::from_fd()
    ///
    /// width: The width for the buffer
    ///
    /// height: The height for the buffer
    ///
    /// format: The fourcc code for the surface
    ///
    /// usage: The union of the usage flags for this buffer
    ///
    /// # Returns
    ///
    /// A newly allocated buffer. If an error occurs during allocation ```None``` will be
    /// returned and errno set.
    ///
    /// # Example
    /// ```ignore
    /// let buffer = BufferObject::new(&device, 1920, 1080,
    ///                                gbm::format::XRGB8888, // GBM_FORMAT_XRGB8888
    ///                                gbm::USE_SCANOUT | gbm::USE_RENDERING);
    /// ```
    ///                                
    pub fn new(dev: &Device, width: u32, height: u32,
               format: u32, flags: u32) -> Option<BufferObject> {
        unsafe {
            let bo = gbm_bo_create(dev.ptr, width, height,
                                   format, flags);

            if bo.is_null() {
                return None;
            }

            return Some(BufferObject { ptr: bo, manual: true });
        }
    }

    /// Get the width of the BufferObject
    ///
    /// # Returns
    ///
    /// The width of the allocated BufferObject
    pub fn width(&self) -> u32 {
        unsafe { gbm_bo_get_width(self.ptr) }
    }

    /// Get the height of the BufferObject
    ///
    /// # Returns
    ///
    /// The height of the allocated BufferObject
    pub fn height(&self) -> u32 {
        unsafe { gbm_bo_get_height(self.ptr) }
    }

    /// Get the stride of the BufferObject
    ///
    /// This is calculated by the backend when it does the allocation in
    /// BufferObject::new()
    ///
    /// # Returns
    ///
    /// The stride of the allocated BufferObject in bytes
    pub fn stride(&self) -> u32 {
        unsafe { gbm_bo_get_stride(self.ptr) }
    }

    /// Get the format of the buffer object
    ///
    /// The format of the pixels in the buffer.
    ///
    /// # Returns
    ///
    /// The format of buffer object, as a fourcc code
    pub fn format(&self) -> u32 {
        unsafe { gbm_bo_get_format(self.ptr) }
    }

    /// Get the gbm device used to create the buffer object
    ///
    /// # Returns
    ///
    /// Returns the gbm device with which the buffer object was created
    pub fn device(&self) -> Device {
        unsafe { Device { ptr: gbm_bo_get_device(self.ptr) } }
    }

    /// Get the handle of the buffer object
    ///
    /// This is stored in the platform generic union gbm_bo_handle type. However
    /// the format of this handle is platform specific.
    ///
    /// # Returns
    ///
    /// Returns the handle of the allocated BufferObject as a ```u32```
    pub fn handle_u32(&self) -> u32 {
        unsafe { gbm_bo_get_handle(self.ptr) as u32 }
    }

    /// Get the handle of the buffer object
    ///
    /// This is stored in the platform generic union gbm_bo_handle type. However
    /// the format of this handle is platform specific.
    ///
    /// # Returns
    ///
    /// Returns the handle of the allocated BufferObject as a ```u64```
    pub fn handle_u64(&self) -> u64 {
        unsafe { gbm_bo_get_handle(self.ptr) }
    }

    /// Get the handle of the buffer object
    ///
    /// This is stored in the platform generic union gbm_bo_handle type. However
    /// the format of this handle is platform specific.
    ///
    /// # Returns
    ///
    /// Returns the handle of the allocated BufferObject as a ```i32```
    pub fn handle_i32(&self) -> i32 {
        unsafe { gbm_bo_get_handle(self.ptr) as i32 }
    }

    /// Get the handle of the buffer object
    ///
    /// This is stored in the platform generic union gbm_bo_handle type. However
    /// the format of this handle is platform specific.
    ///
    /// # Returns
    ///
    /// Returns the handle of the allocated BufferObject as a ```i64```
    pub fn handle_i64(&self) -> i64 {
        unsafe { gbm_bo_get_handle(self.ptr) as i64 }
    }

    /// Get the handle of the buffer object
    ///
    /// This is stored in the platform generic union gbm_bo_handle type. However
    /// the format of this handle is platform specific.
    ///
    /// # Returns
    ///
    /// Returns the handle of the allocated BufferObject as a ```void *```
    pub fn handle_ptr(&self) -> *const c_void {
        unsafe { gbm_bo_get_handle(self.ptr) as *const c_void }
    }

    /// Get a DMA-BUF file descriptor for the buffer object
    ///
    /// This function creates a DMA-BUF (also known as PRIME) file descriptor
    /// handle for the buffer object. Each call to gbm_bo_get_fd() returns a new
    /// file descriptor and the caller is responsible for closing the file
    /// descriptor.
    ///
    /// # Returns
    ///
    /// Returns a file descriptor referring to the underlying buffer
    pub fn fd(&self) -> RawFd {
        unsafe { gbm_bo_get_fd(self.ptr) }
    }

    /// Write data into the buffer object
    ///
    /// If the buffer object was created with the USE_WRITE flag
    /// this function can used to write data into the buffer object. The
    /// data is copied directly into the object and it's the responsiblity
    /// of the caller to make sure the data represents valid pixel data,
    /// according to the width, height, stride and format of the buffer object.
    ///
    /// # Arguments
    ///
    /// buf: The data to write
    ///
    /// count: The number of bytes to write
    ///
    /// # Returns
    ///
    /// Returns ```true``` on success, otherwise ```false``` is returned an errno set
    pub fn write<T>(&self, buf: *const T, count: usize) -> bool {
        unsafe { gbm_bo_write(self.ptr, buf as *const c_void, count) == 0 }
    }

    /// Returns the gbm_bo for the BufferObject
    ///
    /// # Returns
    ///
    /// A pointer to the gbm_bo used to create the BufferObject.
    pub fn c_struct(&self) -> *const gbm_bo {
        self.ptr
    }
}

impl Drop for BufferObject {
    fn drop(&mut self) {
        unsafe { if self.manual { gbm_bo_destroy(self.ptr) } }
    }
}

/// Buffer is going to be presented to the screen using an API such as KMS
pub const USE_SCANOUT: u32 = (1 << 0);
/// Buffer is going to be used as cursor
pub const USE_CURSOR: u32 = (1 << 1);
/// Buffer is to be used for rendering - for example it is going to be used
/// as the storage for a color buffer
pub const USE_RENDERING: u32 = (1 << 2);
/// Buffer can be used for BufferObject::write. This is guaranteed to work
/// with USE_CURSOR, but may not work for other combinations
pub const USE_WRITE: u32 = (1 << 3);

/// Formats
pub mod format {
    macro_rules! fourcc_code {
        ($a:expr, $b:expr, $c:expr, $d:expr) => {
            ($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24)
        }
    }

    // Color index

    /// [7:0] C
    pub const C8: u32 = fourcc_code!('C', '8', ' ', ' ');

    // 8 bpp RGB

    /// [7:0] R:G:B 3:3:2
    pub const RGB332: u32 = fourcc_code!('R', 'G', 'B', '8');
    /// [7:0] B:G:R 2:3:3
    pub const BGR233: u32 = fourcc_code!('B', 'G', 'R', '8');

    // 16 bpp RGB

    /// [15:0] x:R:G:B 4:4:4:4 little endian
    pub const XRGB4444: u32 = fourcc_code!('X', 'R', '1', '2');
    /// [15:0] x:B:G:R 4:4:4:4 little endian
    pub const XBGR4444: u32 = fourcc_code!('X', 'B', '1', '2');
    /// [15:0] R:G:B:x 4:4:4:4 little endian
    pub const RGBX4444: u32 = fourcc_code!('R', 'X', '1', '2');
    /// [15:0] B:G:R:x 4:4:4:4 little endian
    pub const BGRX4444: u32 = fourcc_code!('B', 'X', '1', '2');

    /// [15:0] A:R:G:B 4:4:4:4 little endian
    pub const ARGB4444: u32 = fourcc_code!('A', 'R', '1', '2');
    /// [15:0] A:B:G:R 4:4:4:4 little endian
    pub const ABGR4444: u32 = fourcc_code!('A', 'B', '1', '2');
    /// [15:0] R:G:B:A 4:4:4:4 little endian
    pub const RGBA4444: u32 = fourcc_code!('R', 'A', '1', '2');
    /// [15:0] B:G:R:A 4:4:4:4 little endian
    pub const BGRA4444: u32 = fourcc_code!('B', 'A', '1', '2');

    /// [15:0] x:R:G:B 1:5:5:5 little endian
    pub const XRGB1555: u32 = fourcc_code!('X', 'R', '1', '5');
    /// [15:0] x:B:G:R 1:5:5:5 little endian
    pub const XBGR1555: u32 = fourcc_code!('X', 'B', '1', '5');
    /// [15:0] R:G:B:x 5:5:5:1 little endian
    pub const RGBX5551: u32 = fourcc_code!('R', 'X', '1', '5');
    /// [15:0] B:G:R:x 5:5:5:1 little endian
    pub const BGRX5551: u32 = fourcc_code!('B', 'X', '1', '5');

    /// [15:0] A:R:G:B 1:5:5:5 little endian
    pub const ARGB1555: u32 = fourcc_code!('A', 'R', '1', '5');
    /// [15:0] A:B:G:R 1:5:5:5 little endian
    pub const ABGR1555: u32 = fourcc_code!('A', 'B', '1', '5');
    /// [15:0] R:G:B:A 5:5:5:1 little endian
    pub const RGBA5551: u32 = fourcc_code!('R', 'A', '1', '5');
    /// [15:0] B:G:R:A 5:5:5:1 little endian
    pub const BGRA5551: u32 = fourcc_code!('B', 'A', '1', '5');

    /// [15:0] R:G:B 5:6:5 little endian
    pub const RGB565: u32 = fourcc_code!('R', 'G', '1', '6');
    /// [15:0] B:G:R 5:6:5 little endian
    pub const BGR565: u32 = fourcc_code!('B', 'G', '1', '6');

    // 24 bpp RGB

    /// [23:0] R:G:B little endian
    pub const RGB888: u32 = fourcc_code!('R', 'G', '2', '4');
    /// [23:0] B:G:R little endian
    pub const BGR888: u32 = fourcc_code!('B', 'G', '2', '4');

    // 32 bpp RGB

    /// [31:0] x:R:G:B 8:8:8:8 little endian
    pub const XRGB8888: u32 = fourcc_code!('X', 'R', '2', '4');
    /// [31:0] x:B:G:R 8:8:8:8 little endian
    pub const XBGR8888: u32 = fourcc_code!('X', 'B', '2', '4');
    /// [31:0] R:G:B:x 8:8:8:8 little endian
    pub const RGBX8888: u32 = fourcc_code!('R', 'X', '2', '4');
    /// [31:0] B:G:R:x 8:8:8:8 little endian
    pub const BGRX8888: u32 = fourcc_code!('B', 'X', '2', '4');

    /// [31:0] A:R:G:B 8:8:8:8 little endian
    pub const ARGB8888: u32 = fourcc_code!('A', 'R', '2', '4');
    /// [31:0] A:B:G:R 8:8:8:8 little endian
    pub const ABGR8888: u32 = fourcc_code!('A', 'B', '2', '4');
    /// [31:0] R:G:B:A 8:8:8:8 little endian
    pub const RGBA8888: u32 = fourcc_code!('R', 'A', '2', '4');
    /// [31:0] B:G:R:A 8:8:8:8 little endian
    pub const BGRA8888: u32 = fourcc_code!('B', 'A', '2', '4');

    /// [31:0] x:R:G:B 2:10:10:10 little endian
    pub const XRGB2101010: u32 = fourcc_code!('X', 'R', '3', '0');
    /// [31:0] x:B:G:R 2:10:10:10 little endian
    pub const XBGR2101010: u32 = fourcc_code!('X', 'B', '3', '0');
    /// [31:0] R:G:B:x 10:10:10:2 little endian
    pub const RGBX1010102: u32 = fourcc_code!('R', 'X', '3', '0');
    /// [31:0] B:G:R:x 10:10:10:2 little endian
    pub const BGRX1010102: u32 = fourcc_code!('B', 'X', '3', '0');

    /// [31:0] A:R:G:B 2:10:10:10 little endian
    pub const ARGB2101010: u32 = fourcc_code!('A', 'R', '3', '0');
    /// [31:0] A:B:G:R 2:10:10:10 little endian
    pub const ABGR2101010: u32 = fourcc_code!('A', 'B', '3', '0');
    /// [31:0] R:G:B:A 10:10:10:2 little endian
    pub const RGBA1010102: u32 = fourcc_code!('R', 'A', '3', '0');
    /// [31:0] B:G:R:A 10:10:10:2 little endian
    pub const BGRA1010102: u32 = fourcc_code!('B', 'A', '3', '0');

    // packed YCbCr

    /// [31:0] Cr0:Y1:Cb0:Y0 8:8:8:8 little endian
    pub const YUYV: u32 = fourcc_code!('Y', 'U', 'Y', 'V');
    /// [31:0] Cb0:Y1:Cr0:Y0 8:8:8:8 little endian
    pub const YVYU: u32 = fourcc_code!('Y', 'V', 'Y', 'U');
    /// [31:0] Y1:Cr0:Y0:Cb0 8:8:8:8 little endian
    pub const UYVY: u32 = fourcc_code!('U', 'Y', 'V', 'Y');
    /// [31:0] Y1:Cb0:Y0:Cr0 8:8:8:8 little endian
    pub const VYUY: u32 = fourcc_code!('V', 'Y', 'U', 'Y');

    /// [31:0] A:Y:Cb:Cr 8:8:8:8 little endian
    pub const AYUV: u32 = fourcc_code!('A', 'Y', 'U', 'V');

    // 2 plane YCbCr
    // index 0 = Y plane, [7:0] Y
    // index 1 = Cr:Cb plane, [15:0] Cr:Cb little endian
    // or
    // index 1 = Cb:Cr plane, [15:0] Cb:Cr little endian

    /// 2x2 subsampled Cr:Cb plane
    pub const NV12: u32 = fourcc_code!('N', 'V', '1', '2');
    /// 2x2 subsampled Cb:Cr plane
    pub const NV21: u32 = fourcc_code!('N', 'V', '2', '1');
    /// 2x1 subsampled Cr:Cb plane
    pub const NV16: u32 = fourcc_code!('N', 'V', '1', '6');
    /// 2x1 subsampled Cb:Cr plane
    pub const NV61: u32 = fourcc_code!('N', 'V', '6', '1');
}

//
// C definitions
//

/// Representation of C opaque structure to use in foreign C functions
#[allow(non_camel_case_types)]
pub enum gbm_device {}

/// Representation of C opaque structure to use in foreign C functions
#[allow(non_camel_case_types)]
pub enum gbm_bo {}

/// Representation of C opaque structure to use in foreign C functions
#[allow(non_camel_case_types)]
pub enum gbm_surface {}

#[link(name = "gbm")]
extern {
    fn gbm_device_get_fd(gbm: *const gbm_device) -> c_int;
    // This function doesn't seem very useful
    // fn gbm_device_get_backend_name(gbm: *const gbm_device) -> *const c_char;
    fn gbm_device_is_format_supported(gbm: *const gbm_device,
                                          format: uint32_t, usage: uint32_t) -> c_int;
    fn gbm_device_destroy(gbm: *const gbm_device);
    fn gbm_create_device(fd: c_int) -> *const gbm_device;
    fn gbm_bo_create(gbm: *const gbm_device,
                         width: uint32_t, height:
                         uint32_t, format: uint32_t, flags: uint32_t) -> *const gbm_bo;
    // TODO
    // fn gbm_bo_import(gbm: *const gbm_device, _type: uint32_t,
    //                  buffer: *const c_void, usage: uint32_t) -> *const gbm_bo;
    fn gbm_bo_get_width(bo: *const gbm_bo) -> uint32_t;
    fn gbm_bo_get_height(bo: *const gbm_bo) -> uint32_t;
    fn gbm_bo_get_stride(bo: *const gbm_bo) -> uint32_t;
    fn gbm_bo_get_format(bo: *const gbm_bo) -> uint32_t;
    fn gbm_bo_get_device(bo: *const gbm_bo) -> *const gbm_device;
    fn gbm_bo_get_handle(bo: *const gbm_bo) -> uint64_t;
    fn gbm_bo_get_fd(bo: *const gbm_bo) -> c_int;
    fn gbm_bo_write(bo: *const gbm_bo, buf: *const c_void, count: size_t) -> c_int;
    // TODO
    // fn gbm_bo_set_user_data(bo: *const gbm_bo, data: *const c_void,
    //                         destroy_user_data: extern fn(bo: *const gbm_bo, data: *const c_void));
    // TODO
    // fn gbm_bo_get_user_data(bo: *const gbm_bo) -> *const c_void;
    fn gbm_bo_destroy(bo: *const gbm_bo);
    fn gbm_surface_create(gbm: *const gbm_device,
                              width: uint32_t, height: uint32_t,
                              format: uint32_t, flags: uint32_t) -> *const gbm_surface;
    // This function doesn't seem to have actually been implemented
    // fn gbm_surface_needs_lock_front_buffer(surface: *const gbm_surface) -> c_int;
    fn gbm_surface_lock_front_buffer(surface: *const gbm_surface) -> *const gbm_bo;
    fn gbm_surface_release_buffer(surface: *const gbm_surface, bo: *const gbm_bo);
    fn gbm_surface_has_free_buffers(surface: *const gbm_surface) -> c_int;
    fn gbm_surface_destroy(surface: *const gbm_surface);
}
