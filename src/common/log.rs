use core::fmt::Write;
use core::slice;

pub trait Logger {
    fn new(buffer: FrameBuffer) -> Self;
    fn log_char(&mut self, char: u32);

    fn log(&mut self, s: &str) {
        let _ = self.try_log(s);
    }

    fn try_log(&mut self, s: &str) -> Result<(), ()> {
        for c in s.chars() {
            let char_num = c as u32;

            self.log_char(char_num);
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FrameBufferInfo {
    pub address: usize,
    pub len: usize,
    pub screen_width: u32,
    pub screen_height: u32,
    pub pixels_per_scan_line: u32,
}

#[derive(Debug)]
pub struct FrameBuffer {
    buffer: *mut u32,
    slice: &'static mut [u32],
    pub info: FrameBufferInfo,
}

impl FrameBuffer {
    #[allow(unsafe_code)]
    pub fn new(info: FrameBufferInfo) -> FrameBuffer {
        let buffer = info.address as *mut u32;
        let slice = unsafe { slice::from_raw_parts_mut(buffer, info.len) };

        FrameBuffer {
            buffer,
            slice,
            info,
        }
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u32) {
        assert!(x < self.info.screen_width);
        assert!(y < self.info.screen_height);

        let pos = y * self.info.pixels_per_scan_line + x;
        self.slice[pos as usize] = color;
    }

    pub fn draw_offset(&mut self, offset_start: u64, offset_end: u64, color: u32) {
        for i in offset_start as usize..offset_end as usize {
            self.slice[i] = color
        }
    }
}

impl Clone for FrameBuffer {
    fn clone(&self) -> Self {
        FrameBuffer::new(self.info.clone())
    }
}
