use std::marker::PhantomData;

use anyhow::Result;
use vello::{
    kurbo::{Affine, Rect},
    peniko::{Brush, Color, Fill},
};
use winit::{
    dpi::PhysicalSize,
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::base::{
    AppContext, AppEvent, AppHandler, AppRenderer, DrawFillRectangleOptions,
    DrawMonospaceTextOptions,
};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

struct Position<T> {
    x: T,
    y: T,
}

pub struct App {
    text: String,
    cursor_pos: Position<u32>,
}

impl AppHandler for App {
    fn handle_events(&mut self, event: AppEvent, screen_size: PhysicalSize<u32>) {
        let AppEvent::KeyboardEvent {
            event,
            is_synthetic,
        } = event;

        // TODO: We have no way of bounding the cursor for the right boundary and bottom boundary
        // because we don't have access to the font metrics

        if matches!(event.state, ElementState::Pressed) {
            match event.physical_key {
                PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.cursor_pos.x = self.cursor_pos.x.saturating_sub(1);
                }
                PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.cursor_pos.y = self.cursor_pos.y.saturating_sub(1);
                }
                PhysicalKey::Code(KeyCode::ArrowRight) => {
                    self.cursor_pos.x += 1;
                }
                PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.cursor_pos.y += 1;
                }
                _ => {}
            }
        }

        self.text = format!("Event: is_synthetic is {}, rest: {:?}", is_synthetic, event);
    }

    fn render(&mut self, renderer: &mut AppRenderer, screen_size: PhysicalSize<u32>) {
        let font_size = 16.0;

        let single_space_width = renderer.get_monospace_bounds(font_size, " ").0 as f64;
        let font_height = renderer.get_monospace_font_height(font_size) as f64;

        renderer.draw_fill_rectangle(DrawFillRectangleOptions {
            x: self.cursor_pos.x as f64 * single_space_width,
            y: self.cursor_pos.y as f64 * font_height,
            width: single_space_width,
            height: font_height,
            fill_color: Color::rgb(0.0, 1.0, 0.0),
        });

        let total_tildes = (screen_size.height as f64 / font_height).ceil() as usize;

        renderer.draw_monospace_text(DrawMonospaceTextOptions::<&Brush, _, _> {
            size: 16.0,
            transform: Affine::translate((0.0, 0.0)),
            glyph_transform: None,
            brush: &Brush::Solid(Color::WHITE),
            style: Fill::NonZero,
            text: "~\n".repeat(total_tildes),
            _marker: PhantomData,
        });

        let message_row = total_tildes / 3;

        renderer.draw_monospace_text(DrawMonospaceTextOptions::<&Brush, _, _> {
            size: 16.0,
            transform: Affine::translate((
                single_space_width * 6.0,
                font_height * (message_row as f64),
            )),
            glyph_transform: None,
            brush: &Brush::Solid(Color::WHITE),
            style: Fill::NonZero,
            text: format!("{APP_NAME} editor -- version {APP_VERSION}"),
            _marker: PhantomData,
        });

        renderer.draw_monospace_text(DrawMonospaceTextOptions::<&Brush, _, _> {
            size: 16.0,
            transform: Affine::translate((single_space_width * 6.0, font_height * 7.0)),
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
        AppContext::new(APP_NAME.to_string()).run(App {
            text: "No events yet!".to_string(),
            cursor_pos: Position { x: 0, y: 0 },
        })
    }
}
