use std::marker::PhantomData;

use vello::{
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};

use crate::base::{AppFont, AppRenderer, DrawTextOptions, Size};

use super::buffer::{buffer_lines, Buffer};

pub struct View {
    buffer: Buffer,
}

impl View {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }

    pub fn render(
        &self,
        renderer: &mut AppRenderer,
        view_size: Size<u32>,
        monospace_font: &AppFont,
        monospace_font_size: f32,
    ) {
        let bounds = monospace_font
            .variations(&[])
            .measure_text(monospace_font_size, " ");
        let font_height = bounds.h as f64;

        let total_rows = ((view_size.h as f32) / bounds.h).floor() as usize;
        let empty_row_text = "~".to_string();

        (0..total_rows).for_each(|r| {
            let text = buffer_lines(&self.buffer)
                .iter()
                .nth(r)
                .unwrap_or_else(|| &empty_row_text);
            renderer.draw_text(DrawTextOptions::<&Brush, _, _> {
                font: monospace_font,
                size: monospace_font_size,
                transform: Affine::translate((0.0f64, (r + 1) as f64 * font_height)),
                glyph_transform: None,
                brush: &Brush::Solid(Color::WHITE),
                style: Fill::NonZero,
                text,
                _marker: PhantomData,
            });
        });
    }
}
