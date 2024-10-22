use std::{env, fs, marker::PhantomData, path::Path};

use vello::{
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};
use winit::{
    event::{
        ElementState,
        MouseScrollDelta::{LineDelta, PixelDelta},
    },
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    base::{
        AppContext, AppEvent, AppFont, AppHandler, AppRenderer, Bounds, DrawFillRectangleOptions,
        DrawTextOptions, Position, Size,
    },
    editor::{Buffer, View},
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
    monospace_font_size: f32,
    text: String,
    cursor_pos: Position<u32>,
    view: View,
}

impl AppHandler for App {
    fn handle_events(&mut self, event: AppEvent, screen_size: Size<u32>) {
        let bounds = self
            .monospace_font
            .variations(&[])
            .measure_text(self.monospace_font_size, "~");
        let max_x = screen_size.w / (bounds.w.ceil() as u32);
        let max_y = screen_size.h / (bounds.h.ceil() as u32);

        match event {
            AppEvent::MouseWheelEvent { delta, .. } => {
                self.text = format!("MouseWheelEvent: {:?}", event);
                let mut scroll_offset = self.view.scroll_offset();

                match delta {
                    LineDelta(right, down) => {
                        scroll_offset.x += bounds.w as f64 * -right as f64;
                        scroll_offset.y += bounds.h as f64 * -down as f64;
                    }
                    PixelDelta(physical_position) => {
                        scroll_offset.x += physical_position.x;
                        scroll_offset.y += physical_position.y;
                    }
                };
                self.view.set_scroll_offset(scroll_offset);
            }
            AppEvent::KeyboardEvent {
                event,
                is_synthetic,
            } => {
                let mut should_adjust_scroll_wrt_cursor = false;
                if matches!(event.state, ElementState::Pressed) {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyH) => {
                            self.cursor_pos.x = self.cursor_pos.x.saturating_sub(1);
                            should_adjust_scroll_wrt_cursor = true;
                        }
                        PhysicalKey::Code(KeyCode::KeyK) => {
                            self.cursor_pos.y = self.cursor_pos.y.saturating_sub(1);
                            should_adjust_scroll_wrt_cursor = true;
                        }
                        PhysicalKey::Code(KeyCode::KeyL) => {
                            self.cursor_pos.x += 1;
                            should_adjust_scroll_wrt_cursor = true;
                        }
                        PhysicalKey::Code(KeyCode::KeyJ) => {
                            self.cursor_pos.y += 1;
                            should_adjust_scroll_wrt_cursor = true;
                        }
                        PhysicalKey::Code(KeyCode::ArrowDown) => {
                            let mut offset = self.view.scroll_offset();
                            offset.y += 1.0;
                            self.view.set_scroll_offset(offset);
                        }
                        PhysicalKey::Code(KeyCode::ArrowUp) => {
                            let mut offset = self.view.scroll_offset();
                            offset.y -= 1.0;
                            self.view.set_scroll_offset(offset);
                        }
                        PhysicalKey::Code(KeyCode::ArrowLeft) => {
                            let mut offset = self.view.scroll_offset();
                            offset.x -= 1.0;
                            self.view.set_scroll_offset(offset);
                        }
                        PhysicalKey::Code(KeyCode::ArrowRight) => {
                            let mut offset = self.view.scroll_offset();
                            offset.x += 1.0;
                            self.view.set_scroll_offset(offset);
                        }
                        _ => {}
                    }
                }

                self.text = format!("Event: is_synthetic is {}, rest: {:?}", is_synthetic, event);

                // TODO: Feel like this logic should be outside?
                if should_adjust_scroll_wrt_cursor {
                    let current_cursor_global_bounds = Bounds {
                        pos: Position {
                            x: self.cursor_pos.x as f64 * bounds.w as f64,
                            y: self.cursor_pos.y as f64 * bounds.h as f64,
                        },
                        size: Size {
                            w: bounds.w as f64,
                            h: bounds.h as f64,
                        },
                    };

                    let current_scroll_offset = self.view.scroll_offset();
                    let current_viewport = self.view.viewport();
                    let current_viewport = Bounds {
                        pos: Position {
                            x: current_viewport.pos.x as f64,
                            y: current_viewport.pos.y as f64,
                        },
                        size: Size {
                            w: current_viewport.size.w as f64,
                            h: current_viewport.size.h as f64,
                        },
                    };

                    let mut current_scroll_viewport_offset = Bounds {
                        pos: Position {
                            x: current_viewport.pos.x + current_scroll_offset.x,
                            y: current_viewport.pos.y + current_scroll_offset.y,
                        },
                        size: current_viewport.size,
                    };
                    if current_scroll_viewport_offset.right() <= current_cursor_global_bounds.left()
                    {
                        current_scroll_viewport_offset.pos.x = current_cursor_global_bounds.right()
                            - current_scroll_viewport_offset.size.w;
                    }
                    if current_cursor_global_bounds.left() <= current_scroll_viewport_offset.left()
                    {
                        current_scroll_viewport_offset.pos.x = current_cursor_global_bounds.left();
                    }
                    if current_scroll_viewport_offset.bottom() <= current_cursor_global_bounds.top()
                    {
                        current_scroll_viewport_offset.pos.y = current_cursor_global_bounds
                            .bottom()
                            - current_scroll_viewport_offset.size.h;
                    }
                    if current_cursor_global_bounds.top() <= current_scroll_viewport_offset.top() {
                        current_scroll_viewport_offset.pos.y = current_cursor_global_bounds.top();
                    }

                    let new_scroll = Position {
                        x: current_scroll_viewport_offset.pos.x - current_viewport.pos.x,
                        y: current_scroll_viewport_offset.pos.y - current_viewport.pos.y,
                    };

                    self.view.set_scroll_offset(new_scroll);
                }
            }
            AppEvent::ResizeEvent { new_size } => {
                self.cursor_pos.x = self.cursor_pos.x.min(max_x);
                self.cursor_pos.y = self.cursor_pos.y.min(max_y);

                self.view.set_viewport(Bounds {
                    pos: Position { x: 0, y: 0 },
                    size: Size {
                        w: new_size.w,
                        h: new_size.h,
                    },
                });
                self.text = format!("Event: Resize to {:?}", new_size);
            }
        }
    }

    fn render(&mut self, renderer: &mut AppRenderer, screen_size: Size<u32>) {
        let bounds = self
            .monospace_font
            .variations(&[])
            .measure_text(self.monospace_font_size, " ");
        let single_space_width = bounds.w as f64;
        let font_height = bounds.h as f64;

        renderer.draw_fill_rectangle(DrawFillRectangleOptions {
            pos: Position {
                x: self.cursor_pos.x as f64 * single_space_width - self.view.scroll_offset().x,
                y: self.cursor_pos.y as f64 * font_height - self.view.scroll_offset().y,
            },
            size: Size {
                w: single_space_width,
                h: font_height,
            },
            fill_color: Color::rgb(0.0, 1.0, 0.0),
        });

        self.view
            .render(renderer, &self.monospace_font, self.monospace_font_size);

        if self.view.buffer_empty() {
            let total_tildes = (screen_size.h as f64 / font_height).ceil() as usize;
            let message_row = total_tildes / 3;

            renderer.draw_text(DrawTextOptions::<&Brush, _, _> {
                font: &self.monospace_font,
                size: self.monospace_font_size,
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
        }

        renderer.draw_text(DrawTextOptions::<&Brush, _, _> {
            font: &self.monospace_font,
            size: self.monospace_font_size,
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
    fn get_filepath_arg() -> Option<String> {
        env::args().nth(1).map(|s| s.to_string())
    }

    pub fn run() {
        let filepath_arg = Self::get_filepath_arg();

        AppContext::new(APP_NAME.to_string()).run(App {
            monospace_font: load_monospace_font(),
            monospace_font_size: 16.0,
            text: "No events yet!".to_string(),
            cursor_pos: Position { x: 0, y: 0 },
            view: View::new(
                filepath_arg
                    .map(|path| {
                        if !fs::exists(&path).unwrap_or(false)
                            || fs::metadata(&path).map(|md| md.is_file()).unwrap_or(false)
                        {
                            Buffer::load(path)
                        } else {
                            // TODO: Handle directory properly
                            Buffer::new()
                        }
                    })
                    .unwrap_or_else(Buffer::new),
                Bounds {
                    pos: Position { x: 0, y: 0 },
                    size: Size { w: 1, h: 1 },
                },
            ),
        });
    }
}
