use std::marker::PhantomData;

use anyhow::Result;
use vello::{
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};
use winit::dpi::PhysicalSize;

use crate::base::{AppContext, AppEvent, AppHandler, AppRenderer, DrawMonospaceTextOptions};

pub struct App {
    text: String,
}

impl AppHandler for App {
    fn handle_events(&mut self, event: AppEvent) {
        // TODO: Handle events
        let AppEvent::KeyboardEvent {
            event,
            is_synthetic,
        } = event;
        self.text = format!("Event: is_synthetic is {}, rest: {:?}", is_synthetic, event);
    }

    fn render(&mut self, renderer: &mut AppRenderer, screen_size: PhysicalSize<u32>) {
        let font_size = 16.0f32;
        let font_height = renderer.get_monospace_font_height(font_size);

        let total_tildes = ((screen_size.height as f32) / font_height).ceil() as usize;

        renderer.draw_monospace_text(DrawMonospaceTextOptions::<&Brush, _, _> {
            size: 16.0,
            transform: Affine::translate((0.0, 0.0)),
            glyph_transform: None,
            brush: &Brush::Solid(Color::WHITE),
            style: Fill::NonZero,
            text: "~\n".repeat(total_tildes),
            _marker: PhantomData,
        });

        renderer.draw_monospace_text(DrawMonospaceTextOptions::<&Brush, _, _> {
            size: 16.0,
            transform: Affine::translate((30.0, font_height as f64 * 3.0)),
            glyph_transform: None,
            brush: &Brush::Solid(Color::WHITE),
            style: Fill::NonZero,
            text: &self.text,
            _marker: PhantomData,
        });
    }
}

impl App {
    pub fn run() -> Result<()> {
        AppContext::new().run(App {
            text: "No events yet!".to_string(),
        })
    }
}
