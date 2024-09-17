use anyhow::Result;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct ActiveAppState {
    window: Arc<Window>,
}

struct SuspendedAppState {
    cached_window: Option<Arc<Window>>,
}

enum AppState {
    Active(ActiveAppState),
    Suspended(SuspendedAppState),
}

pub struct App {
    app_state: AppState,
}

fn create_winit_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(860, 640))
        .with_resizable(true)
        .with_title("wlte");
    Arc::new(event_loop.create_window(attr).unwrap())
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let AppState::Suspended(SuspendedAppState { cached_window }) = &mut self.app_state else {
            return;
        };

        let window = cached_window
            .take()
            .unwrap_or_else(|| create_winit_window(event_loop));

        // TODO: Rendering

        self.app_state = AppState::Active(ActiveAppState { window });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // TODO: Rendering

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            app_state: AppState::Suspended(SuspendedAppState {
                cached_window: None,
            }),
        }
    }
    pub fn run(mut self) -> Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop
            .run_app(&mut self)
            .expect("Cannot run event loop");

        Ok(())
    }
}
