//! Window and Vulkan surface management

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, DeviceEvent, KeyEvent, ElementState},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window as WinitWindow, WindowAttributes, WindowId},
    keyboard::{KeyCode, PhysicalKey},
    dpi::{LogicalSize, PhysicalSize},
};
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};
use lithos_engine_memory::GLOBAL_STATS;
use lithos_engine_logging::{LoggingConfig, init as init_logging};
use lithos_engine_profiler::Profiler;
use lithos_engine_config::{EngineConfig, WindowConfig};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WindowError {
    #[error("Event loop error: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
    #[error("Window creation failed: {0}")]
    WindowCreation(#[from] winit::error::OsError),
    #[error("Vulkan surface creation failed: {0}")]
    SurfaceCreation(String),
}

/// Window state
pub struct WindowState {
    pub window: Arc<WinitWindow>,
    pub config: WindowConfig,
    pub size: PhysicalSize<u32>,
    pub frame_count: u64,
    pub minimized: bool,
    pub focused: bool,
    pub cursor_grabbed: bool,
    pub cursor_visible: bool,
}

impl WindowState {
    pub fn new(event_loop: &ActiveEventLoop, config: &WindowConfig) -> Result<Self, WindowError> {
        let attrs = WindowAttributes::default()
            .with_title(&config.title)
            .with_inner_size(LogicalSize::new(config.width, config.height))
            .with_resizable(config.resizable)
            .with_visible(true);

        let window = Arc::new(event_loop.create_window(attrs)?);

        if config.fullscreen {
            window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }

        let size = window.inner_size();

        Ok(Self {
            window,
            config: config.clone(),
            size,
            frame_count: 0,
            minimized: false,
            focused: true,
            cursor_grabbed: false,
            cursor_visible: true,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
    }

    pub fn set_cursor_grab(&mut self, grabbed: bool) -> Result<(), String> {
        self.cursor_grabbed = grabbed;
        if grabbed {
            self.window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                .or_else(|_| self.window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
                .map_err(|e| e.to_string())?;
            self.window.set_cursor_visible(false);
            self.cursor_visible = false;
        } else {
            self.window.set_cursor_grab(winit::window::CursorGrabMode::None)
                .map_err(|e| e.to_string())?;
            self.window.set_cursor_visible(true);
            self.cursor_visible = true;
        }
        Ok(())
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.config.fullscreen = fullscreen;
        if fullscreen {
            self.window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        } else {
            self.window.set_fullscreen(None);
        }
    }
}

/// Application event handler
pub struct AppHandler {
    window_state: Option<WindowState>,
    config: EngineConfig,
    running: bool,
    event_callback: Option<Box<dyn FnMut(&WindowEvent) + Send>>,
    device_event_callback: Option<Box<dyn FnMut(&DeviceEvent) + Send>>,
}

impl AppHandler {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            window_state: None,
            config,
            running: true,
            event_callback: None,
            device_event_callback: None,
        }
    }

    pub fn set_event_callback(&mut self, cb: impl FnMut(&WindowEvent) + Send + 'static) {
        self.event_callback = Some(Box::new(cb));
    }

    pub fn set_device_event_callback(&mut self, cb: impl FnMut(&DeviceEvent) + Send + 'static) {
        self.device_event_callback = Some(Box::new(cb));
    }

    pub fn window(&self) -> Option<&WindowState> {
        self.window_state.as_ref()
    }

    pub fn window_mut(&mut self) -> Option<&mut WindowState> {
        self.window_state.as_mut()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn request_exit(&mut self) {
        self.running = false;
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_state.is_none() {
            let window = WindowState::new(event_loop, &self.config.window).unwrap();
            self.window_state = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let Some(cb) = &mut self.event_callback {
            cb(&event);
        }

        let window = match &mut self.window_state {
            Some(w) => w,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                self.running = false;
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                window.resize(size);
            }
            WindowEvent::RedrawRequested => {
                window.frame_count += 1;
            }
            WindowEvent::Focused(focused) => {
                window.focused = focused;
            }
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key, state, .. }, .. } => {
                match (physical_key, state) {
                    (PhysicalKey::Code(KeyCode::Escape), ElementState::Pressed) => {
                        if window.cursor_grabbed {
                            window.set_cursor_grab(false).ok();
                        } else {
                            self.running = false;
                            event_loop.exit();
                        }
                    }
                    (PhysicalKey::Code(KeyCode::F11), ElementState::Pressed) => {
                        window.set_fullscreen(!window.config.fullscreen);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: DeviceEvent) {
        if let Some(cb) = &mut self.device_event_callback {
            cb(&event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window_state {
            window.window.request_redraw();
        }
    }
}

/// Run the application
pub fn run_app(config: EngineConfig) -> Result<(), WindowError> {
    init_logging(LoggingConfig::default());
    Profiler::init();

    let event_loop = EventLoop::new()?;
    let mut handler = AppHandler::new(config);

    event_loop.run_app(&mut handler)?;

    Ok(())
}

/// High-level window wrapper for integration with Vulkan
pub struct LithosWindow {
    pub state: WindowState,
    pub surface: Option<ash::vk::SurfaceKHR>,
}

impl LithosWindow {
    pub fn new(event_loop: &ActiveEventLoop, config: &WindowConfig, _instance: &ash::Instance) -> Result<Self, WindowError> {
        let state = WindowState::new(event_loop, config)?;

        // Vulkan surface creation is platform-specific and will be implemented separately
        // For now, return a window without a surface
        Ok(Self {
            state,
            surface: None,
        })
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.state.size
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.state.size.width as f32 / self.state.size.height as f32
    }

    pub fn request_redraw(&self) {
        self.state.window.request_redraw();
    }

    pub fn set_cursor_grab(&mut self, grabbed: bool) -> Result<(), String> {
        self.state.set_cursor_grab(grabbed)
    }
}

impl HasWindowHandle for LithosWindow {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.state.window.window_handle()
    }
}

impl HasDisplayHandle for LithosWindow {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.state.window.display_handle()
    }
}