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

use super::renderer::BaseAppRenderer;

pub trait BaseAppLogic {
    fn handle_events(&mut self, event: BaseAppEvent);
    fn render(&mut self, renderer: &mut BaseAppRenderer);
}

#[derive(Debug)]
pub enum BaseAppEvent {
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

pub struct BaseApp<T: BaseAppLogic> {
    app_state: AppState,
    app_renderer: BaseAppRenderer,
    app_logic: T,
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

impl<T: BaseAppLogic> ApplicationHandler for BaseApp<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let AppState::Suspended(SuspendedAppState { cached_window }) = &mut self.app_state else {
            return;
        };

        let window = cached_window
            .take()
            .unwrap_or_else(|| create_winit_window(event_loop));

        let surface = self.app_renderer.create_vello_surface(&window);

        self.app_state = AppState::Active(ActiveAppState { window, surface });
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let AppState::Active(ActiveAppState { window, .. }) = &self.app_state {
            self.app_state = AppState::Suspended(SuspendedAppState {
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
        let active_state = match &mut self.app_state {
            AppState::Active(state) if state.window.id() == window_id => state,
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.app_renderer
                    .resize_surface(&mut active_state.surface, &size);
            }
            WindowEvent::RedrawRequested => {
                self.app_renderer.start_new_frame();
                self.app_logic.render(&mut self.app_renderer);
                self.app_renderer.present_frame(&active_state.surface);
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                self.app_logic.handle_events(BaseAppEvent::KeyboardEvent {
                    event,
                    is_synthetic,
                });
                active_state.window.request_redraw();
            }
            _ => {}
        }
    }
}

impl<T: BaseAppLogic> BaseApp<T> {
    pub fn new(app_logic: T) -> Self {
        Self {
            app_state: AppState::Suspended(SuspendedAppState {
                cached_window: None,
            }),
            app_renderer: BaseAppRenderer::new(),
            app_logic,
        }
    }
    pub fn run(mut self) -> Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop
            .run_app(&mut self)
            .expect("cannot run event loop");

        Ok(())
    }
}
