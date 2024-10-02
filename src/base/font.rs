use std::{fs, path::Path, sync::Arc};
use vello::{
    glyph::skrifa::FontRef,
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

pub fn to_font_ref(font: &Font) -> Option<FontRef<'_>> {
    use vello::skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}
