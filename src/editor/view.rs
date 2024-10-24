use std::marker::PhantomData;

use unicode_segmentation::UnicodeSegmentation;
use vello::{
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};

use crate::base::{AppFont, AppRenderer, Bounds, DrawTextOptions, Position};

use super::buffer::{buffer_lines, Buffer};

pub struct View {
    buffer: Buffer,
    viewport: Bounds<u32>,
    scroll_offset: Position<f64>,
}

impl View {
    pub fn new(buffer: Buffer, viewport: Bounds<u32>) -> Self {
        Self {
            buffer,
            viewport,
            scroll_offset: Position { x: 0.0, y: 0.0 },
        }
    }

    pub fn buffer_empty(&self) -> bool {
        buffer_lines(&self.buffer).is_empty()
    }

    pub fn viewport(&self) -> Bounds<u32> {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: Bounds<u32>) {
        self.viewport = viewport;
    }

    pub fn scroll_offset(&self) -> Position<f64> {
        self.scroll_offset
    }

    pub fn set_scroll_offset(&mut self, offset: Position<f64>) {
        self.scroll_offset = offset;
    }

    pub fn line_len_at(&self, line: usize) -> usize {
        buffer_lines(&self.buffer)
            .get(line)
            .map(|line| line.graphemes(true).count())
            .unwrap_or(0)
    }

    pub fn total_lines(&self) -> usize {
        buffer_lines(&self.buffer).len()
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

        let total_text_rows = ((self.viewport.size.h as f32) / bounds.h).ceil() as usize;
        let empty_row_text = "~".to_string();

        let start_line = (self.scroll_offset.y / bounds.h as f64).floor() as usize;
        let start_x = -self.scroll_offset.x;
        let start_y = -(self.scroll_offset.y - (start_line as f64 * bounds.h as f64));

        (0..total_text_rows).for_each(|r| {
            let text = buffer_lines(&self.buffer)
                .get(r + start_line)
                .unwrap_or(&empty_row_text);
            renderer.draw_text(DrawTextOptions::<&Brush, _, _> {
                font: monospace_font,
                size: monospace_font_size,
                transform: Affine::translate((
                    self.viewport.pos.x as f64 + start_x,
                    self.viewport.pos.y as f64 + start_y + (r + 1) as f64 * font_height,
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
