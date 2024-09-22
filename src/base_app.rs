use anyhow::Result;
use std::{fs, marker::PhantomData, num::NonZeroUsize, path::Path, sync::Arc};
use vello::{
    glyph::{
        skrifa::{FontRef, MetadataProvider},
        Glyph,
    },
    kurbo::Affine,
    peniko::{Blob, BrushRef, Color, Font, StyleRef},
    util::{RenderContext, RenderSurface},
    wgpu::{Maintain, PresentMode},
    AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene,
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

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

pub struct BaseAppRenderer {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    // reuse scene every frame, so that we don't spend resources
    // recreating it every frame
    scene: Scene,

    monospace_font: Font,
}

fn load_monospace_font() -> Font {
    let monospace_font_path = if cfg!(windows) {
        Path::new(r"C:\Windows\Fonts\consola.ttf")
    } else {
        panic!("don't know where to find monospace font");
    };

    let monospace_font_bytes = fs::read(monospace_font_path).expect("fail to load monospace font");

    Font::new(Blob::new(Arc::new(monospace_font_bytes)), 0)
}

fn to_font_ref(font: &Font) -> Option<FontRef<'_>> {
    use vello::skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}

pub struct DrawMonospaceTextOptions<'a, B, S, T>
where
    B: Into<BrushRef<'a>>,
    S: Into<StyleRef<'a>>,
    T: AsRef<str>,
{
    pub size: f32,
    pub transform: Affine,
    pub glyph_transform: Option<Affine>,
    pub brush: B,
    pub style: S,
    pub text: T,
    pub _marker: PhantomData<&'a ()>,
}

impl BaseAppRenderer {
    fn new() -> Self {
        Self {
            context: RenderContext::new(),
            renderers: vec![],
            scene: Scene::new(),
            monospace_font: load_monospace_font(),
        }
    }

    // our window is backed by an Arc, so we actually can use static lifetime for RenderSurface
    fn create_vello_surface(&mut self, window: &Arc<Window>) -> RenderSurface<'static> {
        let size = window.inner_size();

        // wgpu may crash if width or height is 0, don't allow that
        let width = size.width.max(1);
        let height = size.height.max(1);

        let surface_future =
            self.context
                .create_surface(window.clone(), width, height, PresentMode::AutoVsync);
        let surface = pollster::block_on(surface_future).expect("error creating surface");

        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

        surface
    }

    fn resize_surface(&self, surface: &mut RenderSurface, size: &PhysicalSize<u32>) {
        // wgpu may crash if width or height is 0, don't allow that
        let width = size.width.max(1);
        let height = size.height.max(1);

        self.context.resize_surface(surface, width, height);
    }

    fn start_new_frame(&mut self) {
        self.scene.reset();
    }

    fn present_frame(&mut self, surface: &RenderSurface) {
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

    pub fn draw_monospace_text<'a, B, S, T>(
        &'a mut self,
        options: DrawMonospaceTextOptions<'a, B, S, T>,
    ) where
        B: Into<BrushRef<'a>>,
        S: Into<StyleRef<'a>>,
        T: AsRef<str>,
    {
        let font_ref = to_font_ref(&self.monospace_font).expect("cannot get font ref");

        let axes = font_ref.axes();
        let font_size = vello::skrifa::instance::Size::new(options.size);
        // TODO: Support customising font axes
        let variations: &[(&str, f32)] = &[];
        let var_loc = axes.location(variations.iter().copied());
        let metrics = font_ref.metrics(font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let charmap = font_ref.charmap();
        let glyph_metrics = font_ref.glyph_metrics(font_size, &var_loc);

        let mut pen_x = 0f32;
        let mut pen_y = 0f32;

        self.scene
            .draw_glyphs(&self.monospace_font)
            .font_size(options.size)
            .transform(options.transform)
            .glyph_transform(options.glyph_transform)
            .brush(options.brush)
            .hint(false)
            .draw(
                options.style,
                options.text.as_ref().chars().filter_map(|ch| {
                    if ch == '\n' {
                        pen_y += line_height;
                        pen_x = 0.0;
                        return None;
                    }

                    let gid = charmap.map(ch).unwrap_or_default();
                    let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                    let x = pen_x;
                    pen_x += advance;
                    Some(Glyph {
                        id: gid.to_u32(),
                        x,
                        y: pen_y,
                    })
                }),
            );
    }
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
            .expect("Cannot run event loop");

        Ok(())
    }
}
