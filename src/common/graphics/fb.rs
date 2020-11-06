//! Framebuffer for the OS.
use alloc::vec::Vec;
use num_traits::float::Float;
use rusttype::{point, Point};
use tui::{backend::Backend, layout::Rect, style::Color};

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

    pub fn put_raw_pixel(&mut self, point: Point<i32>, c: (u8, u8, u8)) {
        let index: u32 = (point.x + (point.y * self.scanline as i32)) as u32;
        let val: u32 = (c.0 as u32) << 16 as u32 | (c.1 as u32) << 8 as u32 | (c.2 as u32) as u32;

        if index < self.double_buffer.len() as u32 {
            self.double_buffer[index as usize] = val;
        } else {
            warn!(
                "Pixel failed.. {:?} {:?} {:?}",
                point,
                self.scanline,
                self.double_buffer.len()
            );
        }
    }
}

impl<'a> Backend for FrameBrufferDisplay<'a> {
    fn draw<'b, I>(&mut self, content: I) -> Result<(), tui::io::Error>
    where
        I: Iterator<Item = (u16, u16, &'b tui::buffer::Cell)>,
    {
        let default_fg = convert_color(Color::White).unwrap();
        let default_bg = convert_color(Color::Black).unwrap();

        for (cx, cy, cell) in content {
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

            let fg = convert_color(cell.fg).unwrap_or(default_fg);
            let bg = convert_color(cell.bg).unwrap_or(default_bg);

            if let Some(bb) = positioned.pixel_bounding_box() {
                positioned.draw(|x, y, v| {
                    // v should be in the range 0.0 to 1.0
                    let r = (v * fg.0 as f32) + (1.0 - v) * (bg.0 as f32);
                    let g = (v * fg.1 as f32) + (1.0 - v) * (bg.1 as f32);
                    let b = (v * fg.2 as f32) + (1.0 - v) * (bg.2 as f32);
                    let color = (r.ceil() as u8, g.ceil() as u8, b.ceil() as u8);

                    let x = x as i32 + bb.min.x;
                    let y = y as i32 + bb.min.y;

                    let x = (self.font_cache.cell_width * cx) as i32 + x;
                    let y = (self.font_cache.cell_height * cy) as i32 + y;

                    self.put_raw_pixel(point(x, y), color);
                });
            } else {
                for x in 0..self.font_cache.cell_width {
                    for y in 0..self.font_cache.cell_height {
                        let x: i32 = (self.font_cache.cell_width * cx) as i32 + x as i32;
                        let y: i32 = (self.font_cache.cell_height * cy) as i32 + y as i32;

                        self.put_raw_pixel(point(x, y), bg);
                    }
                }
            }
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
        self.fb[..].copy_from_slice(&self.double_buffer);
        Ok(())
    }
}

fn convert_color(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Reset => None,
        Color::Black => Some((0x00, 0x00, 0x00)),
        Color::Red => Some((0xFF, 0x00, 0x00)),
        Color::Green => Some((0x00, 0xFF, 0x00)),
        Color::Yellow => Some((0xFF, 0xFF, 0x00)),
        Color::Blue => Some((0x00, 0x00, 0xFF)),
        Color::Magenta => Some((0xFF, 0x00, 0xFF)),
        Color::Cyan => Some((0x00, 0xFF, 0xFF)),
        Color::Gray => Some((0x80, 0x80, 0x80)),
        Color::DarkGray => Some((0xA9, 0xA9, 0xA9)),
        Color::LightRed => Some((0xFF, 0xCC, 0xCB)),
        Color::LightGreen => Some((0x90, 0xEE, 0x90)),
        Color::LightYellow => Some((0xFF, 0xFF, 0xE0)),
        Color::LightBlue => Some((0xAD, 0xD8, 0xE6)),
        Color::LightMagenta => Some((0xFF, 0x80, 0xFF)),
        Color::LightCyan => Some((0xE0, 0xFF, 0xFF)),
        Color::White => Some((0xFF, 0xFF, 0xFF)),
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Indexed(_) => None,
    }
}
