use std::marker::PhantomData;

use anyhow::Result;
use vello::{
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};

use crate::base_app::{
    BaseApp, BaseAppEvent, BaseAppLogic, BaseAppRenderer, DrawMonospaceTextOptions,
};

struct AppLogic;

impl BaseAppLogic for AppLogic {
    fn handle_events(&mut self, event: BaseAppEvent) {
        // TODO: Handle events
        println!("{:?}", event);
    }

    fn render(&mut self, renderer: &mut BaseAppRenderer) {
        // TODO: Handle rendering
        renderer.draw_monospace_text(DrawMonospaceTextOptions::<&Brush, _, _> {
            size: 16.0,
            transform: Affine::translate((30.0, 50.0)),
            glyph_transform: None,
            brush: &Brush::Solid(Color::WHITE).into(),
            style: Fill::NonZero,
            text: "Hello world!",
            _marker: PhantomData,
        });
    }
}

pub struct App {
    base_app: BaseApp<AppLogic>,
}

impl App {
    pub fn new() -> Self {
        Self {
            base_app: BaseApp::new(AppLogic),
        }
    }

    pub fn run(self) -> Result<()> {
        self.base_app.run()
    }
}
