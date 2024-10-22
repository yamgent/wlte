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

    // if current line is shorter than previous line, remember the previous line x pos so that
    // advancing further will not cause us to lose the x pos
    //
    // for example, suppose the cursor is |, and it is currently in this position:
    //
    //     The quick brown fox
    //     jumps over
    //     the very lazy do|g.
    //
    // if we move up a line, the cursor x position will be shifted because "jumps over" is shorter
    // than "the very lazy dog".
    //
    //     The quick brown fox
    //     jumps over|
    //     the very lazy dog.
    //
    // with the "cursor_previous_line_x_pos" set to the same position as "d|og" and remembered, if
    // we move the cursor up again, then it would get back to the correct x pos:
    //
    //     The quick brown f|ox
    //     jumps over
    //     the very lazy dog.
    //
    // WITHOUT "cursor_previous_line_x_pos", it would become like this instead:
    //
    //     The quick |brown fox
    //     jumps over
    //     the very lazy dog.
    //
    //  which is undesirable behaviour.
    cursor_previous_line_x_pos: Option<u32>,
    cursor_pos: Position<u32>,
    view: View,
}

#[derive(Clone, Copy, Debug)]
enum ScrollCommand {
    Px(f64, f64),
    Cells(i64, i64),
}

#[derive(Clone, Copy, Debug)]
enum Command {
    MoveCursorLeftWrap,
    MoveCursorRightWrap,
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorToStartOfLine,
    MoveCursorToEndOfLine,
    MoveCursorUpOneViewPage,
    MoveCursorDownOneViewPage,
    ScrollView(ScrollCommand),
}

impl App {
    fn get_single_cell_size(&self) -> Size<f32> {
        self.monospace_font
            .variations(&[])
            .measure_text(self.monospace_font_size, "~")
    }

    fn execute_command(&mut self, command: Command) {
        let previous_cursor_pos = self.cursor_pos;

        match command {
            Command::MoveCursorLeftWrap => {
                if self.cursor_pos.x == 0 {
                    if self.cursor_pos.y != 0 {
                        self.cursor_pos.y -= 1;
                        self.cursor_pos.x = self
                            .view
                            .line_len_at(self.cursor_pos.y as usize)
                            .saturating_sub(1) as u32;
                    }
                } else {
                    self.cursor_pos.x = self.cursor_pos.x - 1;
                }
            }
            Command::MoveCursorRightWrap => {
                if self.cursor_pos.x + 1 >= self.view.line_len_at(self.cursor_pos.y as usize) as u32
                {
                    if self.cursor_pos.y + 1 != self.view.total_lines() as u32 {
                        self.cursor_pos.y += 1;
                        self.cursor_pos.x = 0;
                    }
                } else {
                    self.cursor_pos.x = self.cursor_pos.x + 1;
                }
            }
            Command::MoveCursorUp => {
                self.cursor_pos.y = self.cursor_pos.y.saturating_sub(1);
            }
            Command::MoveCursorDown => {
                self.cursor_pos.y = (self.cursor_pos.y + 1).min(self.view.total_lines() as u32 - 1);
            }
            Command::MoveCursorToStartOfLine => {
                self.cursor_pos.x = 0;
            }
            Command::MoveCursorToEndOfLine => {
                self.cursor_pos.x = self
                    .view
                    .line_len_at(self.cursor_pos.y as usize)
                    .saturating_sub(1) as u32;
            }
            Command::MoveCursorUpOneViewPage => {
                let cell_size = self.get_single_cell_size();
                self.cursor_pos.y = self
                    .cursor_pos
                    .y
                    .saturating_sub((self.view.viewport().size.h as f32 / cell_size.h) as u32);
            }
            Command::MoveCursorDownOneViewPage => {
                let cell_size = self.get_single_cell_size();
                self.cursor_pos.y = (self.cursor_pos.y
                    + (self.view.viewport().size.h as f32 / cell_size.h) as u32)
                    .min(self.view.total_lines().saturating_sub(1) as u32);
            }
            Command::ScrollView(ScrollCommand::Px(x_px, y_px)) => {
                let mut scroll_offset = self.view.scroll_offset();
                scroll_offset.x += x_px;
                scroll_offset.y += y_px;
                self.view.set_scroll_offset(scroll_offset);
            }
            Command::ScrollView(ScrollCommand::Cells(x_cell, y_cell)) => {
                let cell_size = self.get_single_cell_size();

                let mut scroll_offset = self.view.scroll_offset();
                scroll_offset.x += cell_size.w as f64 * x_cell as f64;
                scroll_offset.y += cell_size.h as f64 * y_cell as f64;
                self.view.set_scroll_offset(scroll_offset);
            }
        }

        if matches!(
            command,
            Command::MoveCursorUp
                | Command::MoveCursorDown
                | Command::MoveCursorUpOneViewPage
                | Command::MoveCursorDownOneViewPage
        ) {
            let current_line_len = self.view.line_len_at(self.cursor_pos.y as usize) as u32;

            if self.cursor_previous_line_x_pos.is_none() {
                self.cursor_previous_line_x_pos = Some(previous_cursor_pos.x);
            }

            if self.cursor_pos.x >= current_line_len {
                self.cursor_pos.x = current_line_len.saturating_sub(1);
            } else if let Some(prev_pos) = self.cursor_previous_line_x_pos {
                if prev_pos < current_line_len {
                    self.cursor_pos.x = prev_pos;
                } else {
                    self.cursor_pos.x = current_line_len.saturating_sub(1);
                }
            }
        } else {
            self.cursor_previous_line_x_pos.take();
        }

        let should_adjust_scroll_wrt_cursor = matches!(
            command,
            Command::MoveCursorLeftWrap
                | Command::MoveCursorRightWrap
                | Command::MoveCursorUp
                | Command::MoveCursorDown
                | Command::MoveCursorToStartOfLine
                | Command::MoveCursorToEndOfLine
                | Command::MoveCursorUpOneViewPage
                | Command::MoveCursorDownOneViewPage
        );

        if should_adjust_scroll_wrt_cursor {
            let cell_size = self.get_single_cell_size();
            let current_cursor_global_bounds = Bounds {
                pos: Position {
                    x: self.cursor_pos.x as f64 * cell_size.w as f64,
                    y: self.cursor_pos.y as f64 * cell_size.h as f64,
                },
                size: Size {
                    w: cell_size.w as f64,
                    h: cell_size.h as f64,
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
            if current_scroll_viewport_offset.right() <= current_cursor_global_bounds.right() {
                current_scroll_viewport_offset.pos.x =
                    current_cursor_global_bounds.right() - current_scroll_viewport_offset.size.w;
            }
            if current_cursor_global_bounds.left() <= current_scroll_viewport_offset.left() {
                current_scroll_viewport_offset.pos.x = current_cursor_global_bounds.left();
            }
            if current_scroll_viewport_offset.bottom() <= current_cursor_global_bounds.bottom() {
                current_scroll_viewport_offset.pos.y =
                    current_cursor_global_bounds.bottom() - current_scroll_viewport_offset.size.h;
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
}

impl AppHandler for App {
    fn handle_events(&mut self, event: AppEvent, screen_size: Size<u32>) {
        let bounds = self.get_single_cell_size();
        let max_x = screen_size.w / (bounds.w.ceil() as u32);
        let max_y = screen_size.h / (bounds.h.ceil() as u32);

        match event {
            AppEvent::MouseWheelEvent { delta, .. } => {
                self.text = format!("MouseWheelEvent: {:?}", event);
                match delta {
                    LineDelta(right, down) => self.execute_command(Command::ScrollView(
                        ScrollCommand::Cells(-right as i64, -down as i64),
                    )),
                    PixelDelta(physical_position) => self.execute_command(Command::ScrollView(
                        ScrollCommand::Px(physical_position.x, physical_position.y),
                    )),
                };
            }
            AppEvent::KeyboardEvent {
                event,
                is_synthetic,
            } => {
                if matches!(event.state, ElementState::Pressed) {
                    let commands = match event.physical_key {
                        PhysicalKey::Code(KeyCode::ArrowDown) => {
                            vec![Command::ScrollView(ScrollCommand::Px(0.0, 1.0))]
                        }
                        PhysicalKey::Code(KeyCode::ArrowUp) => {
                            vec![Command::ScrollView(ScrollCommand::Px(0.0, -1.0))]
                        }
                        PhysicalKey::Code(KeyCode::ArrowLeft) => {
                            vec![Command::ScrollView(ScrollCommand::Px(-1.0, 0.0))]
                        }
                        PhysicalKey::Code(KeyCode::ArrowRight) => {
                            vec![Command::ScrollView(ScrollCommand::Px(1.0, 0.0))]
                        }
                        PhysicalKey::Code(KeyCode::Home) => {
                            vec![Command::MoveCursorToStartOfLine]
                        }
                        PhysicalKey::Code(KeyCode::End) => {
                            vec![Command::MoveCursorToEndOfLine]
                        }
                        PhysicalKey::Code(KeyCode::PageUp) => {
                            vec![Command::MoveCursorUpOneViewPage]
                        }
                        PhysicalKey::Code(KeyCode::PageDown) => {
                            vec![Command::MoveCursorDownOneViewPage]
                        }
                        _ => {
                            if let Some(ref text) = event.text {
                                match text.as_ref() {
                                    "$" => {
                                        vec![Command::MoveCursorToEndOfLine]
                                    }
                                    "0" => {
                                        vec![Command::MoveCursorToStartOfLine]
                                    }
                                    "h" => {
                                        vec![Command::MoveCursorLeftWrap]
                                    }
                                    "j" => {
                                        vec![Command::MoveCursorDown]
                                    }
                                    "k" => {
                                        vec![Command::MoveCursorUp]
                                    }
                                    "l" => {
                                        vec![Command::MoveCursorRightWrap]
                                    }
                                    _ => {
                                        vec![]
                                    }
                                }
                            } else {
                                vec![]
                            }
                        }
                    };
                    commands
                        .into_iter()
                        .for_each(|command| self.execute_command(command));
                }

                self.text = format!("Event: is_synthetic is {}, rest: {:?}", is_synthetic, event);
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
            cursor_previous_line_x_pos: None,
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
