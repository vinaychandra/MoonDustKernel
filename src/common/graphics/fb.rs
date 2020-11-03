//! Framebuffer for the OS.
use alloc::vec::Vec;
use rusttype::{point, Point};
use tui::{backend::Backend, buffer::Cell, layout::Rect};

use super::fonts::FontCache;

pub struct FrameBrufferDisplay<'a> {
    fb: &'a mut [u32],
    double_buffer: Vec<u32>,
    bb: Rect,
    pub cursor: (u16, u16),
    font_cache: FontCache<'a>,
    scanline: u32,
}

impl<'a> FrameBrufferDisplay<'a> {
    pub fn new(fb: &'a mut [u32], size: Rect, scanline: u32) -> FrameBrufferDisplay {
        let db: Vec<u32> = vec![0; fb.len()];
        FrameBrufferDisplay {
            fb,
            bb: size,
            double_buffer: db,
            cursor: (0, 0),
            font_cache: FontCache::new(),
            scanline,
        }
    }

    pub fn put_raw_pixel(&mut self, point: Point<i32>, value: u32) {
        let index: u32 = (point.x + (point.y * self.scanline as i32)) as u32;

        if index < self.double_buffer.len() as u32 {
            self.double_buffer[index as usize] = value;
        } else {
            warn!(
                "Pixel failed.. {:?} {:?} {:?}",
                point,
                self.scanline,
                self.double_buffer.len()
            );
        }
    }

    fn do_draw(&mut self, cx: u16, cy: u16, cell: &Cell) {
        let positioned = {
            let glyph = self.font_cache.regular_font.glyph(
                cell.symbol
                    .chars()
                    .into_iter()
                    .next()
                    .expect("Expected atleast one char"),
            );
            let scaled = glyph.scaled(self.font_cache.scale);
            scaled.positioned(self.font_cache.regular_offset)
        };

        let mapping_scale = 255 as f32; // 8 bit color
        if let Some(bb) = positioned.pixel_bounding_box() {
            positioned.draw(|x, y, v| {
                // v should be in the range 0.0 to 1.0
                let i = (v * mapping_scale + 0.5) as u8 as u32;
                let gray = i | i << 8 | i << 16;

                let x = x as i32 + bb.min.x;
                let y = y as i32 + bb.min.y;

                let x = (self.font_cache.cell_width * cx) as i32 + x;
                let y = (self.font_cache.cell_height * cy) as i32 + y;

                self.put_raw_pixel(point(x, y), gray);
            });
        } else {
            // warn!("No pixel bounding box for this {:?}", cell);
            let _ = 1;
        }
    }
}

impl<'a> Backend for FrameBrufferDisplay<'a> {
    fn draw<'b, I>(&mut self, content: I) -> Result<(), tui::io::Error>
    where
        I: Iterator<Item = (u16, u16, &'b tui::buffer::Cell)>,
    {
        for (x, y, c) in content {
            self.do_draw(x, y, c);
        }

        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), tui::io::Error> {
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), tui::io::Error> {
        Ok(())
    }

    fn get_cursor(&mut self) -> Result<(u16, u16), tui::io::Error> {
        Ok(self.cursor)
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> Result<(), tui::io::Error> {
        self.cursor = (x, y);
        Ok(())
    }

    fn clear(&mut self) -> Result<(), tui::io::Error> {
        for val in &mut self.double_buffer {
            *val = 0;
        }

        Ok(())
    }

    fn size(&self) -> Result<tui::layout::Rect, tui::io::Error> {
        Ok(self.bb)
    }

    fn flush(&mut self) -> Result<(), tui::io::Error> {
        info!("Flushing to screen");
        self.fb[..].copy_from_slice(&self.double_buffer);
        Ok(())
    }
}
