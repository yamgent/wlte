use std::sync::Arc;
use vello::util::RenderSurface;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceId, KeyEvent, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use super::{
    renderer::{AppRenderer, BaseAppRenderer},
    Size,
};

pub trait AppHandler {
    fn handle_events(&mut self, event: AppEvent, screen_size: Size<u32>);
    fn render(&mut self, renderer: &mut AppRenderer, screen_size: Size<u32>);
}

#[derive(Debug)]
pub enum AppEvent {
    KeyboardEvent {
        event: KeyEvent,
        is_synthetic: bool,
    },
    MouseWheelEvent {
        device_id: DeviceId,
        delta: MouseScrollDelta,
        phase: TouchPhase,
    },
    ResizeEvent {
        new_size: Size<u32>,
    },
}

struct ActiveAppState {
    // our window is backed by an Arc, so we actually can use static lifetime for RenderSurface
    surface: RenderSurface<'static>,
    window: Arc<Window>,
}

struct SuspendedAppState {
    cached_window: Option<Arc<Window>>,
}

enum AppState {
    Active(ActiveAppState),
    Suspended(SuspendedAppState),
}

struct BaseApp<T: AppHandler> {
    state: AppState,
    renderer: BaseAppRenderer,
    handler: T,
    name: String,
}

fn create_winit_window<T: AsRef<str>>(
    event_loop: &ActiveEventLoop,
    window_title: T,
) -> Arc<Window> {
    let attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(860, 640))
        .with_resizable(true)
        .with_title(window_title.as_ref().to_string());
    Arc::new(
        event_loop
            .create_window(attr)
            .expect("cannot create window"),
    )
}

impl<T: AppHandler> ApplicationHandler for BaseApp<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let AppState::Suspended(SuspendedAppState { cached_window }) = &mut self.state else {
            return;
        };

        let window = cached_window
            .take()
            .unwrap_or_else(|| create_winit_window(event_loop, &self.name));

        let surface = self.renderer.create_vello_surface(&window);

        self.state = AppState::Active(ActiveAppState { window, surface });
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let AppState::Active(ActiveAppState { window, .. }) = &self.state {
            self.state = AppState::Suspended(SuspendedAppState {
                cached_window: Some(window.clone()),
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // only handle event if it is our window, and we are in active state
        let active_state = match &mut self.state {
            AppState::Active(state) if state.window.id() == window_id => state,
            _ => return,
        };

        let surface_size = Size {
            w: active_state.surface.config.width,
            h: active_state.surface.config.height,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.renderer
                    .resize_surface(&mut active_state.surface, &size);

                let screen_size = Size {
                    w: size.width,
                    h: size.height,
                };

                self.handler.handle_events(
                    AppEvent::ResizeEvent {
                        new_size: screen_size,
                    },
                    screen_size,
                );
            }
            WindowEvent::RedrawRequested => {
                self.renderer.start_new_frame();

                self.handler
                    .render(&mut ((&mut self.renderer).into()), surface_size);
                self.renderer.present_frame(&active_state.surface);
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                self.handler.handle_events(
                    AppEvent::KeyboardEvent {
                        event,
                        is_synthetic,
                    },
                    surface_size,
                );
                active_state.window.request_redraw();
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                self.handler.handle_events(
                    AppEvent::MouseWheelEvent {
                        device_id,
                        delta,
                        phase,
                    },
                    surface_size,
                );
                active_state.window.request_redraw();
            }
            _ => {}
        }
    }
}

pub struct AppContext {
    state: AppState,
    renderer: BaseAppRenderer,
    name: String,
}

impl AppContext {
    pub fn new(name: String) -> Self {
        Self {
            state: AppState::Suspended(SuspendedAppState {
                cached_window: None,
            }),
            renderer: BaseAppRenderer::new(),
            name,
        }
    }

    pub fn run(self, handler: impl AppHandler) {
        let event_loop = EventLoop::new().expect("cannot create event loop");
        event_loop
            .run_app(&mut BaseApp {
                state: self.state,
                renderer: self.renderer,
                name: self.name,
                handler,
            })
            .expect("cannot run event loop");
    }
}
