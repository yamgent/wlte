use std::{fs, path::Path, sync::Arc};
use vello::{
    glyph::skrifa::{
        charmap::Charmap,
        instance::Location,
        metrics::{GlyphMetrics, Metrics},
        FontRef, GlyphId, MetadataProvider,
    },
    peniko::{Blob, Font},
};

pub fn load_monospace_font() -> Font {
    let monospace_font_path = if cfg!(windows) {
        Path::new(r"C:\Windows\Fonts\consola.ttf")
    } else {
        panic!("don't know where to find monospace font");
    };

    let monospace_font_bytes = fs::read(monospace_font_path).expect("fail to load monospace font");

    Font::new(Blob::new(Arc::new(monospace_font_bytes)), 0)
}

fn to_font_ref(font: &Font) -> Option<FontRef<'_>> {
    use vello::skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}

pub struct FontMetrics<'a> {
    metrics: Metrics,
    glyph_metrics: GlyphMetrics<'a>,
}

impl<'a> FontMetrics<'a> {
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

pub struct FontGlyphs<'a> {
    charmap: Charmap<'a>,
}

impl<'a> FontGlyphs<'a> {
    fn new(font_ref: &FontRef<'a>) -> Self {
        let charmap = font_ref.charmap();

        Self { charmap }
    }

    pub fn glyph(&self, ch: char) -> GlyphId {
        self.charmap.map(ch).unwrap_or_default()
    }
}

pub struct FontMetadata<'a> {
    font_ref: FontRef<'a>,
    var_loc: Location,
}

impl<'a> FontMetadata<'a> {
    pub fn new(font: &'a Font, variations: &[(&str, f32)]) -> Self {
        let font_ref = to_font_ref(&font).expect("cannot get font ref");
        let var_loc = font_ref.axes().location(variations.iter().copied());

        Self { font_ref, var_loc }
    }

    pub fn glyphs(&self) -> FontGlyphs {
        FontGlyphs::new(&self.font_ref)
    }

    pub fn metrics(&self, font_size: f32) -> FontMetrics {
        FontMetrics::new(&self.font_ref, font_size, &self.var_loc)
    }
}
