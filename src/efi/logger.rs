use crate::common::log::{FrameBuffer, Logger};

const FONT: &[u8] = include_bytes!("../../resources/uni.psf");

const NEW_LINE: u32 = 10;
const CARRIAGE_RETURN: u32 = 13;

pub struct EfiLogger {
    /// The frame buffer for the graphical output
    buffer: FrameBuffer,
    /// The current pixel position on the horizontal line of the frame buffer
    pos_x: u32,
    /// The current pixel position on the vertical line of the frame buffer
    pos_y: u32,
    /// Max chars per line
    chars_per_line: u32,
}

impl Logger for EfiLogger {
    fn new(buffer: FrameBuffer) -> Self {
        let chars_per_line = buffer.info.screen_width / 8;

        EfiLogger {
            buffer,
            pos_x: 0,
            pos_y: 0,
            chars_per_line,
        }
    }

    fn log_char(&mut self, char: u32) {
        match char {
            NEW_LINE => {
                self.pos_y += 16;
            }
            CARRIAGE_RETURN => {
                self.pos_x = 0;
            }
            _ => {
                let pos_char = self.pos_x / 8;

                if pos_char >= self.chars_per_line {
                    //Truncate overflow
                    return;
                }

                let char_start = 4 + 16 * char as usize;
                let char_end = char_start + 16;
                let char_data = &FONT[char_start..char_end];

                for y in 0..16 {
                    for x in 0..8usize {
                        let x_rev = 7 - x;
                        let char_offset = y * 8 + x_rev;
                        let byte = char_offset / 8;
                        let bit = char_offset % 8;

                        let bit_set = char_data[byte] & (1 << bit) == (1 << bit);

                        let color = if bit_set { 0xFFFFFF } else { 0 };

                        self.buffer
                            .draw_pixel(self.pos_x + x as u32, self.pos_y + y as u32, color);
                    }
                }

                self.pos_x += 8
            }
        }
    }
}
