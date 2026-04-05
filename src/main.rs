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
use std::time::{Instant, Duration};

type IcedElement<'a> = Element<'a, Message, Theme, IcedRenderer>;

fn main() -> Result<(), winit::error::EventLoopError> {
    // Inicjalizacja systemu logowania (tracing)
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
    // Wywoływane, gdy aplikacja zostaje wznowiona lub uruchomiona
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attrs = WindowAttributes::default()
                .with_title("Onyx")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Nie udało się utworzyć okna"),
            );

            // Inicjalizacja stanu renderowania (asynchronicznie)
            match pollster::block_on(RenderState::new(window)) {
                Ok(render_state) => {
                    self.state = Some(render_state);
                }
                Err(e) => {
                    tracing::error!("Inicjalizacja grafiki nie powiodła się: {:?}", e);
                    event_loop.exit();
                }
            }
        }
    }

    // Obsługa zdarzeń okna
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

        // Konwersja zdarzeń winit na zdarzenia zrozumiałe dla biblioteki Iced
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
                // Aktualizacja interfejsu użytkownika
                state.update_ui(&mut self.controls);
                
                // Aktualizacja licznika FPS w tytule okna
                state.update_fps_title();

                // Renderowanie ramki
                if let Err(e) = state.render(&mut self.controls) {
                    match e {
                        wgpu::SurfaceError::Lost => state.resize(state.window.inner_size()),
                        wgpu::SurfaceError::OutOfMemory => event_loop.exit(),
                        _ => tracing::error!("Błąd renderowania: {:?}", e),
                    }
                }

                // Zapytanie o ponowne odrysowanie (pętla renderowania)
                state.window.request_redraw();
            }

            _ => {}
        }
    }
}

// ======================= RenderState (Stan Renderowania) =======================

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

    // Pola do obsługi pomiaru klatek na sekundę (FPS)
    last_fps_update: Instant,
    frame_count: u32,
}

impl RenderState {
    async fn new(window: Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;

        // Wybór adaptera graficznego (preferowana wysoka wydajność)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        // Żądanie urządzenia logicznego i kolejki poleceń
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Urządzenie renderujące Onyx"),
                ..Default::default()
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];

        // Konfiguracja powierzchni renderowania
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Immediate, // Tryb Immediate odblokowuje FPS (brak V-Sync)
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let viewport = Viewport::with_physical_size(
            Size::new(size.width, size.height),
            window.scale_factor() as f32,
        );

        // Inicjalizacja silnika Iced
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
            last_fps_update: Instant::now(),
            frame_count: 0,
        })
    }

    /// Przelicza FPS i aktualizuje tytuł okna co sekundę
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

    /// Obsługa zmiany rozmiaru okna
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

    /// Przetwarzanie zdarzeń interfejsu i aktualizacja jego stanu logicznego
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

        // Przekazanie zdarzeń do interfejsu Iced
        interface.update(
            &self.events,
            self.cursor,
            &mut self.iced_renderer,
            &mut messages,
        );

        self.events.clear();
        self.cache = interface.into_cache();

        // Przetworzenie wiadomości zwrotnych z interfejsu
        for msg in messages {
            controls.update(msg);
        }
    }

    /// Renderowanie końcowej klatki obrazu
    fn render(&mut self, controls: &mut Controls) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Główny enkoder renderowania"),
        });

        // Czyszczenie tła
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Czyszczenie tła"),
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

        // Wysłanie poleceń czyszczenia do kolejki
        self.queue.submit(std::iter::once(encoder.finish()));

        // Budowa i rysowanie interfejsu Iced
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
        
        // Wyświetlenie narysowanego interfejsu na ekranie
        self.iced_renderer.present(None, self.format, &view, &self.viewport);

        frame.present();
        Ok(())
    }
}

// ======================= Interface (Interfejs i Logika) =======================

#[derive(Debug, Clone)]
pub enum Message {}

pub struct Controls;

impl Controls {
    pub fn new() -> Self {
        Self
    }

    pub fn update(&mut self, _message: Message) {
        // Tutaj trafia obsługa logiki po kliknięciu przycisków itp.
    }

    pub fn view(&self) -> IcedElement<'_> {
        // Definicja wyglądu interfejsu
        container(text("Onyx Engine").size(32).color([0.8, 0.8, 0.8]))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}