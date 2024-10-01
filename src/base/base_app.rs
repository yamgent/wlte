use anyhow::Result;
use std::sync::Arc;
use vello::util::RenderSurface;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use super::renderer::{AppRenderer, BaseAppRenderer};

pub trait AppHandler {
    fn handle_events(&mut self, event: AppEvent);
    fn render(&mut self, renderer: &mut AppRenderer);
}

#[derive(Debug)]
pub enum AppEvent {
    KeyboardEvent { event: KeyEvent, is_synthetic: bool },
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
}

fn create_winit_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(860, 640))
        .with_resizable(true)
        .with_title("wlte");
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
            .unwrap_or_else(|| create_winit_window(event_loop));

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

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.renderer
                    .resize_surface(&mut active_state.surface, &size);
            }
            WindowEvent::RedrawRequested => {
                self.renderer.start_new_frame();
                self.handler.render(&mut ((&mut self.renderer).into()));
                self.renderer.present_frame(&active_state.surface);
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                self.handler.handle_events(AppEvent::KeyboardEvent {
                    event,
                    is_synthetic,
                });
                active_state.window.request_redraw();
            }
            _ => {}
        }
    }
}

pub struct AppContext {
    state: AppState,
    renderer: BaseAppRenderer,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            state: AppState::Suspended(SuspendedAppState {
                cached_window: None,
            }),
            renderer: BaseAppRenderer::new(),
        }
    }

    pub fn run(self, handler: impl AppHandler) -> Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop
            .run_app(&mut BaseApp {
                state: self.state,
                renderer: self.renderer,
                handler,
            })
            .expect("cannot run event loop");

        Ok(())
    }
}
