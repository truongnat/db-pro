//! Isolated Welcome renderer spike. Run with `--features rs-ui-spike`.

#[path = "../rs_ui_welcome_scene.rs"]
mod scene;

use rs_ui_core::Size;
use rs_ui_renderer::{RenderFrame, RendererOptions, UiRenderer, Viewport};
use rs_ui_window::{UiWindow, WindowConfig, WindowMetrics};
use scene::WelcomeScene;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

type SpikeResult<T> = Result<T, Box<dyn std::error::Error>>;

struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: UiRenderer,
    suspended: bool,
}

impl Gpu {
    fn new(window: &UiWindow) -> SpikeResult<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.window().clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("welcome-spike"),
            ..Default::default()
        }))?;
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .ok_or("surface has no sRGB format")?;
        let alpha_mode = if capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            wgpu::CompositeAlphaMode::Opaque
        };
        if !capabilities.alpha_modes.contains(&alpha_mode) {
            return Err("surface has no premultiplied or opaque alpha mode".into());
        }
        let metrics = window.metrics();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: metrics.physical_size[0].max(1),
            height: metrics.physical_size[1].max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let renderer = UiRenderer::new(&device, &queue, format, RendererOptions::default())?;
        tracing::info!(
            "welcome-spike adapter={:?} backend={:?} format={format:?} alpha={alpha_mode:?} msaa=4",
            adapter.get_info().name,
            adapter.get_info().backend
        );
        let mut gpu = Self {
            surface,
            device,
            queue,
            config,
            renderer,
            suspended: false,
        };
        gpu.resize(metrics);
        Ok(gpu)
    }

    fn resize(&mut self, metrics: WindowMetrics) {
        self.config.width = metrics.physical_size[0].max(1);
        self.config.height = metrics.physical_size[1].max(1);
        self.surface.configure(&self.device, &self.config);
        self.renderer.resize(&self.device, metrics.physical_size);
        tracing::info!(
            "welcome-spike logical={}x{} physical={}x{} scale={}",
            metrics.logical_size.width,
            metrics.logical_size.height,
            metrics.physical_size[0],
            metrics.physical_size[1],
            metrics.scale_factor.get()
        );
    }

    fn draw(&mut self, window: &UiWindow, scene: &mut WelcomeScene) -> SpikeResult<()> {
        if self.suspended {
            return Ok(());
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.resize(window.metrics());
                window.request_redraw();
                return Ok(());
            }
            issue => return Err(format!("surface frame unavailable: {issue:?}").into()),
        };
        let metrics = window.metrics();
        scene.layout(metrics.logical_size)?;
        let display_list = scene.tree.paint(scene.root)?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.renderer.render(
            RenderFrame {
                device: &self.device,
                queue: &self.queue,
                encoder: &mut encoder,
                target: &view,
                viewport: Viewport::new(metrics.physical_size, metrics.scale_factor),
            },
            &display_list,
            Some(&mut scene.text),
        );
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        let stats = self.renderer.stats();
        tracing::info!(
            "welcome-spike commands={} draws={} glyphs_rasterized={} atlas={}px",
            display_list.commands().len(),
            stats.draw_calls,
            stats.glyphs_rasterized,
            stats.atlas_used_pixels
        );
        Ok(())
    }
}

struct WelcomeSpike {
    size: Size,
    scene: WelcomeScene,
    window: Option<UiWindow>,
    gpu: Option<Gpu>,
    failure: Option<Box<dyn std::error::Error>>,
}

impl ApplicationHandler for WelcomeSpike {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let initialized = UiWindow::open(
            event_loop,
            &WindowConfig {
                title: "DB Pro — rs-ui Welcome spike".to_owned(),
                logical_size: self.size,
            },
        )
        .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
        .and_then(|window| Gpu::new(&window).map(|gpu| (window, gpu)));
        match initialized {
            Ok((window, gpu)) => {
                window.request_redraw();
                self.window = Some(window);
                self.gpu = Some(gpu);
            }
            Err(error) => {
                self.failure = Some(error);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let (Some(window), Some(gpu)) = (&mut self.window, &mut self.gpu) else {
            return;
        };
        if window.id() != id {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == winit::event::ElementState::Pressed
                    && event.logical_key == winit::keyboard::NamedKey::Escape =>
            {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                gpu.suspended = size.width == 0 || size.height == 0;
                if !gpu.suspended {
                    window.refresh_metrics();
                    gpu.resize(window.metrics());
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                window.refresh_metrics();
                gpu.resize(window.metrics());
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = gpu.draw(window, &mut self.scene) {
                    self.failure = Some(error);
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

fn main() -> SpikeResult<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .try_init()
        .map_err(|error| -> Box<dyn std::error::Error> { error })?;
    let size = match std::env::var("DB_PRO_WINDOW_SIZE") {
        Ok(raw) => {
            let (width, height) = raw.split_once('x').ok_or("DB_PRO_WINDOW_SIZE must be WIDTHxHEIGHT")?;
            let size = Size::new(width.parse()?, height.parse()?);
            if !size.width.is_finite() || !size.height.is_finite() || size.width < 640.0 || size.height < 480.0 {
                return Err("spike viewport must be finite and at least 640x480 logical points".into());
            }
            size
        }
        Err(std::env::VarError::NotPresent) => Size::new(1280.0, 800.0),
        Err(error) => return Err(error.into()),
    };
    let mut app = WelcomeSpike {
        size,
        scene: WelcomeScene::new()?,
        window: None,
        gpu: None,
        failure: None,
    };
    EventLoop::new()?.run_app(&mut app)?;
    match app.failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
