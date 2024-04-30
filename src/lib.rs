pub mod app;
pub mod renderer;

use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::event_loop::EventLoop;
use winit::window::{CursorGrabMode, WindowBuilder};

use std::time::Instant;

const TITLE: &str = "Voxers";

pub async fn run() -> anyhow::Result<()> {
    let env = env_logger::Env::default().filter_or("VOXERS_LOG", "debug");

    env_logger::Builder::from_env(env)
        .filter_module("wgpu_core", log::LevelFilter::Info)
        .filter_module("wgpu_hal", log::LevelFilter::Info)
        .filter_module("naga", log::LevelFilter::Info)
        .init();

    let event_loop = EventLoop::new()?;
    let builder = WindowBuilder::new().with_title(TITLE);
    let window = builder.build(&event_loop)?;

    let window = &*Box::leak(Box::new(window));
    window.set_cursor_visible(false);
    let unmove_cursor = window.set_cursor_grab(CursorGrabMode::Locked).is_err();

    let mut state = app::ApplicationState::new(window).await;

    let size = state.renderer.size;
    let center = PhysicalPosition::new(size.width / 2, size.height / 2);
    window.set_cursor_position(center).unwrap();

    let mut last = Instant::now();
    event_loop.run(move |e_event, elwt| {
        if state.exit {
            elwt.exit();
        }

        let now = Instant::now();
        let dt = now - last;
        last = now;

        state.update(dt);

        match e_event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == window.id() => {
                state.draw();
            }
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                if !state.mouse_input(&event, dt) {
                    match event {
                        WindowEvent::Resized(new_size) => state.renderer.resize(new_size),
                        WindowEvent::KeyboardInput { event, .. } => {
                            state.key_input(&event, dt);
                        }
                        _ => {}
                    }
                }
            }
            Event::DeviceEvent { event, .. } => {
                // keep the cursor centered
                if unmove_cursor && matches!(event, DeviceEvent::MouseMotion { .. }) {
                    let size = state.renderer.size;
                    let center = PhysicalPosition::new(size.width / 2, size.height / 2);
                    window.set_cursor_position(center).unwrap();
                }

                state.mouse_movement(&event, dt);
            }
            _ => {}
        }
    })?;

    Ok(())
}
