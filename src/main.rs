use iced::{
    Element, Length, Theme, mouse,
    widget::{container, text},
};

use iced_wgpu::{
    Engine, Renderer as IcedRenderer,
    graphics::{Shell, Viewport},
};

use iced_winit::{
    conversion,
    core::{Event, Size, renderer},
    runtime::{UserInterface, user_interface},
    winit::{
        self,
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        keyboard::ModifiersState,
        window::{Window, WindowAttributes, WindowId},
    },
};

use std::sync::Arc;
use std::time::{Instant, Duration}; // Dodane dla obsługi czasu

type IcedElement<'a> = Element<'a, Message, Theme, IcedRenderer>;

fn main() -> Result<(), winit::error::EventLoopError> {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new()?;
    let mut app = OnyxApp::new();

    event_loop.run_app(&mut app)
}

struct OnyxApp {
    state: Option<RenderState>,
    controls: Controls,
}

impl OnyxApp {
    fn new() -> Self {
        Self {
            state: None,
            controls: Controls::new(),
        }
    }
}

impl ApplicationHandler for OnyxApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attrs = WindowAttributes::default()
                .with_title("Onyx")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Failed to create window"),
            );

            match pollster::block_on(RenderState::new(window)) {
                Ok(render_state) => {
                    self.state = Some(render_state);
                }
                Err(e) => {
                    tracing::error!("Graphics initialization failed: {:?}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        if let Some(iced_event) = conversion::window_event(
            event.clone(),
            state.window.scale_factor() as f32,
            state.modifiers,
        ) {
            state.events.push(iced_event);
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(new_size) => {
                state.resize(new_size);
            }

            WindowEvent::ModifiersChanged(new_modifiers) => {
                state.modifiers = new_modifiers.state();
            }

            WindowEvent::CursorMoved { position, .. } => {
                state.cursor = mouse::Cursor::Available(conversion::cursor_position(
                    position,
                    state.window.scale_factor() as f32,
                ));
            }

            WindowEvent::RedrawRequested => {
                state.update_ui(&mut self.controls);
                
                // --- AKTUALIZACJA FPS W TYTULE OKNA ---
                state.update_fps_title();

                if let Err(e) = state.render(&mut self.controls) {
                    match e {
                        wgpu::SurfaceError::Lost => state.resize(state.window.inner_size()),
                        wgpu::SurfaceError::OutOfMemory => event_loop.exit(),
                        _ => tracing::error!("Render error: {:?}", e),
                    }
                }

                state.window.request_redraw();
            }

            _ => {}
        }
    }
}

// ======================= RenderState =======================

struct RenderState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    format: wgpu::TextureFormat,
    viewport: Viewport,
    iced_renderer: IcedRenderer,
    cache: user_interface::Cache,
    cursor: mouse::Cursor,
    events: Vec<Event>,
    modifiers: ModifiersState,

    // Nowe pola dla FPS
    last_fps_update: Instant,
    frame_count: u32,
}

impl RenderState {
    async fn new(window: Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Onyx Render Device"),
                ..Default::default()
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Immediate, // Użyj Immediate dla odblokowania FPS
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let viewport = Viewport::with_physical_size(
            Size::new(size.width, size.height),
            window.scale_factor() as f32,
        );

        let engine = Engine::new(
            &adapter,
            device.clone(),
            queue.clone(),
            format,
            None,
            Shell::headless(),
        );

        let iced_renderer = IcedRenderer::new(engine, renderer::Settings::default());

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            format,
            viewport,
            iced_renderer,
            cache: user_interface::Cache::new(),
            cursor: mouse::Cursor::Unavailable,
            events: Vec::new(),
            modifiers: ModifiersState::default(),
            // Inicjalizacja czasu
            last_fps_update: Instant::now(),
            frame_count: 0,
        })
    }

    /// Funkcja przeliczająca FPS i aktualizująca pasek tytułowy
    fn update_fps_title(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_fps_update.elapsed();

        if elapsed >= Duration::from_secs(1) {
            let fps = self.frame_count as f32 / elapsed.as_secs_f32();
            self.window.set_title(&format!("Onyx - FPS: {:.1}", fps));
            
            self.last_fps_update = Instant::now();
            self.frame_count = 0;
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.viewport = Viewport::with_physical_size(
                Size::new(new_size.width, new_size.height),
                self.window.scale_factor() as f32,
            );
        }
    }

    fn update_ui(&mut self, controls: &mut Controls) {
        if self.events.is_empty() {
            return;
        }

        let mut messages = Vec::new();
        let mut interface = UserInterface::build(
            controls.view(),
            self.viewport.logical_size(),
            std::mem::take(&mut self.cache),
            &mut self.iced_renderer,
        );

        interface.update(
            &self.events,
            self.cursor,
            &mut self.iced_renderer,
            &mut messages,
        );

        self.events.clear();
        self.cache = interface.into_cache();

        for msg in messages {
            controls.update(msg);
        }
    }

    fn render(&mut self, controls: &mut Controls) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02, g: 0.02, b: 0.03, a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        let mut interface = UserInterface::build(
            controls.view(),
            self.viewport.logical_size(),
            std::mem::take(&mut self.cache),
            &mut self.iced_renderer,
        );

        interface.draw(
            &mut self.iced_renderer,
            &Theme::Dark,
            &renderer::Style::default(),
            self.cursor,
        );

        self.cache = interface.into_cache();
        self.iced_renderer.present(None, self.format, &view, &self.viewport);

        frame.present();
        Ok(())
    }
}

// ======================= Interface =======================

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Controls;

impl Controls {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _message: Message) {}

    pub fn view(&self) -> IcedElement<'_> {
        container(text("Onyx Engine").size(32).color([0.8, 0.8, 0.8]))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}