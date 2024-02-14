use crate::efi::{EfiGuid, EfiStatus};
use core::slice;

pub const GRAPHICS_OUTPUT_GUID: EfiGuid = EfiGuid::new(
    0x9042a9de,
    0x23dc,
    0x4a38,
    [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
);

#[repr(C)]
pub struct GraphicsOutput {
    query_mode: unsafe extern "efiapi" fn() -> EfiStatus,
    set_mode: unsafe extern "efiapi" fn(
        graphics_output: *const GraphicsOutput,
        mode_number: u32,
    ) -> EfiStatus,
    blt: unsafe extern "efiapi" fn() -> EfiStatus,

    mode: *const GraphicsOutputMode,
}

#[allow(unsafe_code)]
impl GraphicsOutput {
    pub fn mode(&self) -> &GraphicsOutputMode {
        unsafe { &*self.mode }
    }

    pub fn set_mode(&self, mode: u32) -> Result<(), EfiStatus> {
        unsafe {
            let status = (self.set_mode)(self, mode);

            if status == 0 {
                Ok(())
            } else {
                Err(status)
            }
        }
    }

    pub fn draw(&self, offset_start: u64, offset_end: u64, r: u32, g: u32, b: u32) {
        unsafe {
            (*self.mode).draw_offset(offset_start, offset_end, r << 16 + g << 8 + b);
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct GraphicsOutputMode {
    pub max_mode: u32,
    pub mode: u32,
    pub info: *const GraphicsOutputModeInfo,
    pub info_len: u64,
    pub framebuffer: u64,
    pub framebuffer_len: u64,
}

#[allow(unsafe_code)]
impl GraphicsOutputMode {
    pub fn draw_offset(&self, offset_start: u64, offset_end: u64, color: u32) {
        unsafe {
            let framebuffer = slice::from_raw_parts_mut(
                self.framebuffer as *mut u32,
                self.framebuffer_len as usize,
            );

            for i in offset_start as usize..offset_end as usize {
                framebuffer[i] = color
            }
        }
    }

    pub fn draw_pixel(&self, x: u32, y: u32, color: u32) {
        let mode = self.current_mode();

        assert!(x < mode.horizontal_resolution);
        assert!(y < mode.vertical_resolution);

        let framebuffer = unsafe {
            slice::from_raw_parts_mut(self.framebuffer as *mut u32, self.framebuffer_len as usize)
        };

        let pos = y * mode.pixels_per_scan_line + x;
        framebuffer[pos as usize] = color;
    }

    pub fn current_mode(&self) -> &GraphicsOutputModeInfo {
        &self.modes()[self.mode as usize]
    }

    pub fn modes(&self) -> &[GraphicsOutputModeInfo] {
        unsafe { slice::from_raw_parts(self.info, self.info_len as usize) }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct GraphicsOutputModeInfo {
    version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pixel_format: PixelFormat,
    pixel_info: PixelInfo,
    pub pixels_per_scan_line: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct PixelFormat(u32);

impl PixelFormat {
    const PIXEL_RED_GREEN_BLUE_RESERVED_8BIT_PER_COLOR: PixelFormat = PixelFormat(0);
    const PIXEL_BLUE_GREEN_RED_RESERVED_8BIT_PER_COLOR: PixelFormat = PixelFormat(1);
    const PIXEL_BIT_MASK: PixelFormat = PixelFormat(2);
    const PIXEL_BLT_ONLY: PixelFormat = PixelFormat(3);
    const PIXEL_FORMAT_MAX: PixelFormat = PixelFormat(4);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct PixelInfo {
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    reserved_mask: u32,
}
