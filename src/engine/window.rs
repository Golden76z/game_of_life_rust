use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use egui_wgpu::{Renderer as EguiRenderer, ScreenDescriptor};
use egui_winit::State as EguiWinitState;
use pollster::block_on;
use std::sync::Arc;

use crate::engine::renderer::WgpuState;
use crate::ui::ui_counter;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<WgpuState>,
    egui_ctx: Option<egui::Context>,
    egui_state: Option<EguiWinitState>,
    egui_renderer: Option<EguiRenderer>,
    counter: i32,
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

        let egui_ctx = egui::Context::default();
        let egui_state = EguiWinitState::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = EguiRenderer::new(
            gpu.device(),
            gpu.surface_format(),
            egui_wgpu::RendererOptions::default(),
        );

        window.request_redraw();
        self.window = Some(window);
        self.gpu = Some(gpu);
        self.egui_ctx = Some(egui_ctx);
        self.egui_state = Some(egui_state);
        self.egui_renderer = Some(egui_renderer);
        println!("Window + wgpu initialized");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(window), Some(egui_state)) = (&self.window, &mut self.egui_state) {
            let response = egui_state.on_window_event(window, &event);
            if response.repaint {
                window.request_redraw();
            }
            if response.consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(size.width, size.height);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let (
                    Some(window),
                    Some(gpu),
                    Some(egui_ctx),
                    Some(egui_state),
                    Some(egui_renderer),
                ) = (
                    &self.window,
                    &mut self.gpu,
                    &self.egui_ctx,
                    &mut self.egui_state,
                    &mut self.egui_renderer,
                ) {
                    let raw_input = egui_state.take_egui_input(window);
                    let full_output = egui_ctx.run(raw_input, |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.heading("Counter");
                            ui_counter(ui, &mut self.counter);
                            ui.label("This is a label");
                            ui.hyperlink("https://github.com/emilk/egui");
                            ui.text_edit_singleline(&mut "test");
                            if ui.button("Click me").clicked() {}
                            ui.add(egui::Slider::new(&mut 0.0, 0.0..=100.0));
                            ui.add(egui::DragValue::new(&mut 0.0));

                            ui.checkbox(&mut true, "Checkbox");
                        });
                    });

                    egui_state.handle_platform_output(window, full_output.platform_output);
                    let paint_jobs =
                        egui_ctx.tessellate(full_output.shapes, egui_ctx.pixels_per_point());

                    match gpu.render_with(|device, queue, encoder, view| {
                        let size = window.inner_size();
                        let screen_desc = ScreenDescriptor {
                            size_in_pixels: [size.width, size.height],
                            pixels_per_point: window.scale_factor() as f32,
                        };

                        for (id, delta) in &full_output.textures_delta.set {
                            egui_renderer.update_texture(device, queue, *id, delta);
                        }

                        egui_renderer.update_buffers(
                            device,
                            queue,
                            encoder,
                            &paint_jobs,
                            &screen_desc,
                        );

                        {
                            let mut rpass = encoder
                                .begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("egui-pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view,
                                        resolve_target: None,
                                        depth_slice: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    occlusion_query_set: None,
                                    timestamp_writes: None,
                                })
                                .forget_lifetime();

                            egui_renderer.render(&mut rpass, &paint_jobs, &screen_desc);
                        }

                        for id in &full_output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }
                    }) {
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

                    if full_output.viewport_output[&egui::ViewportId::ROOT]
                        .repaint_delay
                        .is_zero()
                    {
                        window.request_redraw();
                    }
                }
            }
            _ => (),
        }
    }
}
