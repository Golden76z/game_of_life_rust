use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use std::num::NonZeroU32;
use std::rc::Rc;

use softbuffer::{Context, Surface};

#[derive(Default)]
pub struct App {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
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

        let window = Rc::new(event_loop.create_window(attrs).unwrap());
        let context = Context::new(window.clone()).unwrap();
        let surface = Surface::new(&context, window.clone()).unwrap();
        window.request_redraw(); // force first paint so compositor shows the window
        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        println!("Window created");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(surface) = &mut self.surface {
                        surface
                            .resize(
                                NonZeroU32::new(size.width).unwrap(),
                                NonZeroU32::new(size.height).unwrap(),
                            )
                            .unwrap();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(window), Some(surface)) = (&self.window, &mut self.surface) {
                    let size = window.inner_size();
                    if size.width == 0 || size.height == 0 {
                        return;
                    }

                    surface
                        .resize(
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        )
                        .unwrap();

                    let mut buffer = surface.buffer_mut().unwrap();
                    for pixel in buffer.iter_mut() {
                        *pixel = 0x00202020;
                    }
                    buffer.present().unwrap();
                }
            }
            _ => (),
        }
    }
}
