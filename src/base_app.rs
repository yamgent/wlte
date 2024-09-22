use anyhow::Result;
use std::{num::NonZeroUsize, sync::Arc};
use vello::{
    kurbo::Affine,
    peniko::Color,
    util::{RenderContext, RenderSurface},
    wgpu::{Maintain, PresentMode},
    AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

pub trait BaseAppLogic {
    fn handle_events(&mut self, event: BaseAppEvent);
    fn render(&mut self);
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
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    app_state: AppState,
    // reuse scene every frame, so that we don't spend resources
    // recreating it every frame
    scene: Scene,
    app_logic: T,
}

fn create_winit_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(860, 640))
        .with_resizable(true)
        .with_title("wlte");
    Arc::new(event_loop.create_window(attr).unwrap())
}

fn create_vello_renderer(context: &RenderContext, surface: &RenderSurface) -> Renderer {
    Renderer::new(
        &context.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: AaSupport::all(),
            num_init_threads: NonZeroUsize::new(1),
        },
    )
    .expect("couldn't create renderer")
}

impl<T: BaseAppLogic> ApplicationHandler for BaseApp<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let AppState::Suspended(SuspendedAppState { cached_window }) = &mut self.app_state else {
            return;
        };

        let window = cached_window
            .take()
            .unwrap_or_else(|| create_winit_window(event_loop));

        let surface = self.create_vello_surface(&window);

        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

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
                // wgpu may crash if width or height is 0, don't allow that
                let width = size.width.max(1);
                let height = size.height.max(1);

                self.context
                    .resize_surface(&mut active_state.surface, width, height);
            }
            WindowEvent::RedrawRequested => {
                self.scene.reset();

                self.app_logic.render();

                // TODO: Actual drawing
                self.scene.stroke(
                    &vello::kurbo::Stroke::new(6.0),
                    Affine::IDENTITY,
                    Color::rgb(0.8, 0.8, 0.8),
                    None,
                    &vello::kurbo::Line::new((100.0, 20.0), (400.0, 50.0)),
                );

                let surface = &active_state.surface;

                let width = surface.config.width;
                let height = surface.config.height;
                let device_handle = &self.context.devices[surface.dev_id];

                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("cannot get surface texture");

                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_surface(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.scene,
                        &surface_texture,
                        &RenderParams {
                            base_color: Color::BLACK,
                            width,
                            height,
                            antialiasing_method: AaConfig::Msaa16,
                        },
                    )
                    .expect("failed to render to surface");

                surface_texture.present();

                device_handle.device.poll(Maintain::Poll);
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => self.app_logic.handle_events(BaseAppEvent::KeyboardEvent {
                event,
                is_synthetic,
            }),
            _ => {}
        }
    }
}

impl<T: BaseAppLogic> BaseApp<T> {
    // our window is backed by an Arc, so we actually can use static lifetime for RenderSurface
    fn create_vello_surface(&mut self, window: &Arc<Window>) -> RenderSurface<'static> {
        let size = window.inner_size();

        // wgpu may crash if width or height is 0, don't allow that
        let width = size.width.max(1);
        let height = size.height.max(1);

        let surface_future =
            self.context
                .create_surface(window.clone(), width, height, PresentMode::AutoVsync);
        pollster::block_on(surface_future).expect("error creating surface")
    }
}

impl<T: BaseAppLogic> BaseApp<T> {
    pub fn new(app_logic: T) -> Self {
        Self {
            context: RenderContext::new(),
            renderers: vec![],
            app_state: AppState::Suspended(SuspendedAppState {
                cached_window: None,
            }),
            scene: Scene::new(),
            app_logic,
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
