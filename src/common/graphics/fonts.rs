use rusttype::{point, Font, Point, Scale};

use crate::arch::globals::{GUI_FONT_HEIGHT, GUI_FONT_X_SCALE};

/// Font cache used to provide different fonts.
pub struct FontCache<'a> {
    pub regular_font: Font<'a>,
    pub regular_offset: Point<f32>,

    pub scale: Scale,
}

impl<'a> FontCache<'a> {
    /// Load the fonts data.
    pub fn new() -> FontCache<'a> {
        let regular_font_data = include_bytes!("../../../others/fonts/UbuntuMono-R.ttf");
        let regular_font = Font::try_from_bytes(regular_font_data as &[u8])
            .expect("Error constructing font from bytes");

        // Desired font pixel height
        let height: f32 = GUI_FONT_HEIGHT;

        // 2x scale in x direction to counter the aspect ratio of monospace characters.
        let scale = Scale {
            x: height * GUI_FONT_X_SCALE,
            y: height,
        };

        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). We don't want to clip the text, so we shift
        // it down with an offset when laying it out. v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let regular_v_metrics = regular_font.v_metrics(scale);
        let regular_offset = point(0.0, regular_v_metrics.ascent);

        FontCache {
            regular_font,
            regular_offset,
            scale,
        }
    }
}
