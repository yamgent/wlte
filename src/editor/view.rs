use std::marker::PhantomData;

use vello::{
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};

use crate::base::{AppFont, AppRenderer, Bounds, DrawTextOptions};

use super::buffer::{buffer_lines, Buffer};

pub struct View {
    buffer: Buffer,
    viewport: Bounds<u32>,
}

impl View {
    pub fn new(buffer: Buffer, viewport: Bounds<u32>) -> Self {
        Self { buffer, viewport }
    }

    pub fn buffer_empty(&self) -> bool {
        buffer_lines(&self.buffer).is_empty()
    }

    pub fn set_viewport(&mut self, viewport: Bounds<u32>) {
        self.viewport = viewport;
    }

    pub fn render(
        &self,
        renderer: &mut AppRenderer,
        monospace_font: &AppFont,
        monospace_font_size: f32,
    ) {
        let bounds = monospace_font
            .variations(&[])
            .measure_text(monospace_font_size, " ");
        let font_height = bounds.h as f64;

        let file_name_text = self
            .buffer
            .file_path()
            .clone()
            .unwrap_or("[No Name]".to_string());
        let file_name_text_bounds = monospace_font
            .variations(&[])
            .measure_text(monospace_font_size, &file_name_text);

        renderer.draw_text(DrawTextOptions::<&Brush, _, _> {
            font: monospace_font,
            size: monospace_font_size,
            transform: Affine::translate((
                self.viewport.pos.x as f64 + self.viewport.size.w as f64
                    - file_name_text_bounds.w as f64,
                self.viewport.pos.y as f64 + font_height,
            )),
            glyph_transform: None,
            brush: &Brush::Solid(Color::WHITE),
            style: Fill::NonZero,
            text: file_name_text,
            _marker: PhantomData,
        });

        let total_text_rows = ((self.viewport.size.h as f32) / bounds.h).floor() as usize;
        let empty_row_text = "~".to_string();

        (0..total_text_rows).for_each(|r| {
            let text = buffer_lines(&self.buffer)
                .get(r)
                .unwrap_or(&empty_row_text);
            renderer.draw_text(DrawTextOptions::<&Brush, _, _> {
                font: monospace_font,
                size: monospace_font_size,
                transform: Affine::translate((
                    self.viewport.pos.x as f64,
                    self.viewport.pos.y as f64 + (r + 1) as f64 * font_height,
                )),
                glyph_transform: None,
                brush: &Brush::Solid(Color::WHITE),
                style: Fill::NonZero,
                text,
                _marker: PhantomData,
            });
        });
    }
}
