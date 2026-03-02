use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use pollster::block_on;
use std::sync::Arc;

use crate::engine::renderer::WgpuState;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<WgpuState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attrs = WindowAttributes::default()
            .with_title("Game of Life")
            .with_inner_size(LogicalSize::new(1200.0, 800.0));

        #[cfg(target_os = "linux")]
        {
            use winit::platform::wayland::WindowAttributesExtWayland;
            // Wayland "general" name maps to app_id, which Hyprland rules can match.
            attrs = attrs.with_name("game_of_life_rust", "game_of_life_rust");
        }

        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let gpu = match block_on(WgpuState::new(window.clone())) {
            Ok(gpu) => gpu,
            Err(e) => {
                eprintln!("wgpu init failed: {e}");
                event_loop.exit();
                return;
            }
        };

        window.request_redraw();
        self.window = Some(window);
        self.gpu = Some(gpu);
        println!("Window + wgpu initialized");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(gpu)) = (&self.window, &mut self.gpu) {
                    match gpu.render() {
                        Ok(()) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let size = window.inner_size();
                            gpu.resize(size.width, size.height);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            eprintln!("surface out of memory");
                            event_loop.exit();
                        }
                        Err(wgpu::SurfaceError::Timeout) => {
                            eprintln!("surface timeout");
                        }
                        Err(wgpu::SurfaceError::Other) => {
                            eprintln!("surface error: other");
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
