use std::marker::PhantomData;

use anyhow::Result;
use vello::{
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};

use crate::base::{AppContext, AppHandler, AppRenderer, BaseAppEvent, DrawMonospaceTextOptions};

pub struct App {
    text: String,
}

impl AppHandler for App {
    fn handle_events(&mut self, event: BaseAppEvent) {
        // TODO: Handle events
        let BaseAppEvent::KeyboardEvent {
            event,
            is_synthetic,
        } = event;
        self.text = format!("Event: is_synthetic is {}, rest: {:?}", is_synthetic, event);
    }

    fn render(&mut self, renderer: &mut AppRenderer) {
        // TODO: Handle rendering
        renderer.draw_monospace_text(DrawMonospaceTextOptions::<&Brush, _, _> {
            size: 16.0,
            transform: Affine::translate((30.0, 50.0)),
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
