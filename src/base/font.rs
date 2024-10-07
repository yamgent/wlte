use std::sync::Arc;
use vello::{
    glyph::skrifa::{
        charmap::Charmap,
        instance::Location,
        metrics::{GlyphMetrics, Metrics},
        FontRef, GlyphId, MetadataProvider,
    },
    peniko::{Blob, Font},
};

use super::Size;

fn to_font_ref(font: &Font) -> Option<FontRef<'_>> {
    use vello::skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}

pub struct AppFont {
    font: Font,
}

pub fn get_font(app_font: &AppFont) -> &Font {
    &app_font.font
}

impl From<Vec<u8>> for AppFont {
    fn from(value: Vec<u8>) -> Self {
        Self {
            font: Font::new(Blob::new(Arc::new(value)), 0),
        }
    }
}

impl AppFont {
    pub fn variations(&self, variations: &[(&str, f32)]) -> AppFontVariations {
        AppFontVariations::new(&self.font, variations)
    }
}

pub struct AppFontVariations<'a> {
    font_ref: FontRef<'a>,
    var_loc: Location,
}

impl<'a> AppFontVariations<'a> {
    fn new(font: &'a Font, variations: &[(&str, f32)]) -> Self {
        let font_ref = to_font_ref(font).expect("cannot get font ref");
        let var_loc = font_ref.axes().location(variations.iter().copied());

        Self { font_ref, var_loc }
    }

    pub fn glyphs(&self) -> AppFontGlyphs {
        AppFontGlyphs::new(&self.font_ref)
    }

    pub fn metrics(&self, font_size: f32) -> AppFontMetrics {
        AppFontMetrics::new(&self.font_ref, font_size, &self.var_loc)
    }

    pub fn measure_text<T: AsRef<str>>(&self, font_size: f32, text: T) -> Size<f32> {
        let font_glyphs = self.glyphs();
        let font_metrics = self.metrics(font_size);

        let mut width = 0.0f32;
        let mut height = 0.0f32;

        text.as_ref().lines().for_each(|line| {
            height += font_metrics.glyph_height();
            let mut line_width = 0.0;

            line.chars().for_each(|ch| {
                let gid = font_glyphs.glyph(ch);
                let advance = font_metrics.glyph_width(gid);
                line_width += advance;
            });

            width = width.max(line_width);
        });

        Size {
            w: width,
            h: height,
        }
    }
}

pub struct AppFontMetrics<'a> {
    metrics: Metrics,
    glyph_metrics: GlyphMetrics<'a>,
}

impl<'a> AppFontMetrics<'a> {
    fn new(font_ref: &FontRef<'a>, font_size: f32, var_loc: &'a Location) -> Self {
        let font_size = vello::skrifa::instance::Size::new(font_size);
        let metrics = font_ref.metrics(font_size, var_loc);
        let glyph_metrics = font_ref.glyph_metrics(font_size, var_loc);

        Self {
            metrics,
            glyph_metrics,
        }
    }

    pub fn glyph_height(&self) -> f32 {
        self.metrics.ascent - self.metrics.descent + self.metrics.leading
    }

    pub fn glyph_width(&self, gid: GlyphId) -> f32 {
        self.glyph_metrics.advance_width(gid).unwrap_or_default()
    }
}

pub struct AppFontGlyphs<'a> {
    charmap: Charmap<'a>,
}

impl<'a> AppFontGlyphs<'a> {
    fn new(font_ref: &FontRef<'a>) -> Self {
        let charmap = font_ref.charmap();

        Self { charmap }
    }

    pub fn glyph(&self, ch: char) -> GlyphId {
        self.charmap.map(ch).unwrap_or_default()
    }
}
