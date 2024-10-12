use std::{marker::PhantomData, num::NonZeroUsize, sync::Arc};
use vello::{
    glyph::Glyph,
    kurbo::{Affine, Rect},
    peniko::{BrushRef, Color, Fill, StyleRef},
    util::{RenderContext, RenderSurface},
    wgpu::{Maintain, PresentMode},
    AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene,
};
use winit::{dpi::PhysicalSize, window::Window};

use super::{font::get_font, AppFont, Position, Size};

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

pub struct BaseAppRenderer {
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    // reuse scene every frame, so that we don't spend resources
    // recreating it every frame
    scene: Scene,
}

impl BaseAppRenderer {
    pub fn new() -> Self {
        Self {
            context: RenderContext::new(),
            renderers: vec![],
            scene: Scene::new(),
        }
    }

    // our window is backed by an Arc, so we actually can use static lifetime for RenderSurface
    pub fn create_vello_surface(&mut self, window: &Arc<Window>) -> RenderSurface<'static> {
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

    pub fn resize_surface(&self, surface: &mut RenderSurface, size: &PhysicalSize<u32>) {
        // wgpu may crash if width or height is 0, don't allow that
        let width = size.width.max(1);
        let height = size.height.max(1);

        self.context.resize_surface(surface, width, height);
    }

    pub fn start_new_frame(&mut self) {
        self.scene.reset();
    }

    pub fn present_frame(&mut self, surface: &RenderSurface) {
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
}

pub struct DrawTextOptions<'a, B, S, T>
where
    B: Into<BrushRef<'a>>,
    S: Into<StyleRef<'a>>,
    T: AsRef<str>,
{
    pub font: &'a AppFont,
    pub size: f32,
    pub transform: Affine,
    pub glyph_transform: Option<Affine>,
    pub brush: B,
    pub style: S,
    pub text: T,
    pub _marker: PhantomData<&'a ()>,
}

pub struct DrawFillRectangleOptions {
    pub pos: Position<f64>,
    pub size: Size<f64>,
    pub fill_color: Color,
}

pub struct AppRenderer<'a>(&'a mut BaseAppRenderer);

impl<'a> From<&'a mut BaseAppRenderer> for AppRenderer<'a> {
    fn from(value: &'a mut BaseAppRenderer) -> Self {
        AppRenderer(value)
    }
}

impl<'ar> AppRenderer<'ar> {
    pub fn draw_fill_rectangle(&mut self, options: DrawFillRectangleOptions) {
        let rect = Rect::new(
            options.pos.x,
            options.pos.y,
            options.pos.x + options.size.w,
            options.pos.y + options.size.h,
        );

        self.0.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            options.fill_color,
            None,
            &rect,
        );
    }

    pub fn draw_text<'a, B, S, T>(&'a mut self, options: DrawTextOptions<'a, B, S, T>)
    where
        B: Into<BrushRef<'a>>,
        S: Into<StyleRef<'a>>,
        T: AsRef<str>,
    {
        // TODO: Support customising font axes
        let variations: &[(&str, f32)] = &[];

        let variations = options.font.variations(variations);

        let font_glyphs = variations.glyphs();
        let font_metrics = variations.metrics(options.size);

        let mut pen_x = 0f32;
        let mut pen_y = 0f32;

        self.0
            .scene
            .draw_glyphs(get_font(options.font))
            .font_size(options.size)
            .transform(options.transform)
            .glyph_transform(options.glyph_transform)
            .brush(options.brush)
            .hint(false)
            .draw(
                options.style,
                options.text.as_ref().chars().filter_map(|ch| {
                    if ch == '\n' {
                        pen_y += font_metrics.glyph_height();
                        pen_x = 0.0;
                        return None;
                    }

                    let gid = font_glyphs.glyph(ch);
                    let advance = font_metrics.glyph_width(gid);
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
