use std::{fs, marker::PhantomData, path::Path};

use anyhow::Result;
use vello::{
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};
use winit::{
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::base::{
    AppContext, AppEvent, AppFont, AppHandler, AppRenderer, DrawFillRectangleOptions,
    DrawTextOptions, Position, Size,
};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn load_monospace_font() -> AppFont {
    let monospace_font_path = if cfg!(windows) {
        Path::new(r"C:\Windows\Fonts\consola.ttf")
    } else {
        panic!("don't know where to find monospace font");
    };

    let monospace_font_bytes = fs::read(monospace_font_path).expect("fail to load monospace font");

    monospace_font_bytes.into()
}

pub struct App {
    monospace_font: AppFont,
    text: String,
    cursor_pos: Position<u32>,
}

impl AppHandler for App {
    fn handle_events(&mut self, event: AppEvent, screen_size: Size<u32>) {
        // TODO: This should not be everywhere?
        let font_size = 16.0;
        let bounds = self
            .monospace_font
            .variations(&[])
            .measure_text(font_size, "~");
        let max_x = screen_size.w / (bounds.w.ceil() as u32);
        let max_y = screen_size.h / (bounds.h.ceil() as u32);

        match event {
            AppEvent::KeyboardEvent {
                event,
                is_synthetic,
            } => {
                if matches!(event.state, ElementState::Pressed) {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyH) => {
                            self.cursor_pos.x = self.cursor_pos.x.saturating_sub(1);
                        }
                        PhysicalKey::Code(KeyCode::KeyK) => {
                            self.cursor_pos.y = self.cursor_pos.y.saturating_sub(1);
                        }
                        PhysicalKey::Code(KeyCode::KeyL) => {
                            self.cursor_pos.x = (self.cursor_pos.x + 1).min(max_x);
                        }
                        PhysicalKey::Code(KeyCode::KeyJ) => {
                            self.cursor_pos.y = (self.cursor_pos.y + 1).min(max_y);
                        }
                        _ => {}
                    }
                }

                self.text = format!("Event: is_synthetic is {}, rest: {:?}", is_synthetic, event);
            }
            AppEvent::ResizeEvent { new_size } => {
                self.cursor_pos.x = self.cursor_pos.x.min(max_x);
                self.cursor_pos.y = self.cursor_pos.y.min(max_y);

                self.text = format!("Event: Resize to {:?}", new_size);
            }
        }
    }

    fn render(&mut self, renderer: &mut AppRenderer, screen_size: Size<u32>) {
        let font_size = 16.0;

        let bounds = self
            .monospace_font
            .variations(&[])
            .measure_text(font_size, " ");
        let single_space_width = bounds.w as f64;
        let font_height = bounds.h as f64;

        renderer.draw_fill_rectangle(DrawFillRectangleOptions {
            pos: Position {
                x: self.cursor_pos.x as f64 * single_space_width,
                y: self.cursor_pos.y as f64 * font_height,
            },
            size: Size {
                w: single_space_width,
                h: font_height,
            },
            fill_color: Color::rgb(0.0, 1.0, 0.0),
        });

        let total_tildes = (screen_size.h as f64 / font_height).ceil() as usize;

        renderer.draw_text(DrawTextOptions::<&Brush, _, _> {
            font: &self.monospace_font,
            size: font_size,
            transform: Affine::translate((0.0, 0.0)),
            glyph_transform: None,
            brush: &Brush::Solid(Color::WHITE),
            style: Fill::NonZero,
            text: "~\n".repeat(total_tildes),
            _marker: PhantomData,
        });

        let message_row = total_tildes / 3;

        renderer.draw_text(DrawTextOptions::<&Brush, _, _> {
            font: &self.monospace_font,
            size: font_size,
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

        renderer.draw_text(DrawTextOptions::<&Brush, _, _> {
            font: &self.monospace_font,
            size: font_size,
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
            monospace_font: load_monospace_font(),
            text: "No events yet!".to_string(),
            cursor_pos: Position { x: 0, y: 0 },
        })
    }
}
