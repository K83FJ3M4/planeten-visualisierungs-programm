use std::sync::{Arc, Mutex};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{WindowAttributes, WindowId, Window};
use crate::canvas::Canvas;
use crate::system::System;

pub enum Application {
    Initializing(Arc<Window>),
    Uninitialized(WindowAttributes, EventLoopProxy<Canvas>),
    Initialized(ApplicationState)
}

pub(super) static CONTENT: Mutex<Option<String>> = Mutex::new(None);

pub struct ApplicationState {
    window: Arc<Window>,
    canvas: Canvas,
    last_position: PhysicalPosition<f64>,
    system: System,
    rotating: bool,
    zoom: f32,
    pitch: f32,
    yaw: f32
}

impl ApplicationHandler<Canvas> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Application::Uninitialized(window_attributes, proxy) = self {
            let window = Arc::new(event_loop.create_window(window_attributes.clone()).unwrap());
            let proxy = proxy.clone();
            *self = Application::Initializing(window.clone());
            Application::init(proxy, window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Application::Initialized(state) = self else { return };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(..) => state.window.request_redraw(),
            WindowEvent::MouseWheel { delta, .. } => {
                state.zoom += match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y * 0.1,
                    winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => y as f32 * 0.001
                };
                state.zoom = state.zoom.max(0.1);
                state.window.request_redraw(); 
            }
            WindowEvent::MouseInput { state: element_state, button: MouseButton::Left, .. } => {
                state.rotating = element_state == ElementState::Pressed;
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if !event.state.is_pressed() { return };
                match event.logical_key {
                    Key::Named(NamedKey::Space) => state.system.speed_up(),   
                    Key::Named(NamedKey::Backspace) => state.system.slow_down(),
                    _ => ()
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                if state.rotating {
                    state.pitch -= (position.y - state.last_position.y) as f32;
                    state.yaw += (position.x - state.last_position.x) as f32;

                    state.pitch = state.pitch.clamp(-90.0, 90.0);
                    state.pitch %= 360.0;
                    state.yaw %= 360.0;
                    state.window.request_redraw();
                } 

                if let Some(content) = CONTENT.lock().unwrap().take() {
                    state.system = System::new(&state.canvas.device, content);
                    state.window.request_redraw();
                }

                state.last_position = position;
            }
            WindowEvent::RedrawRequested => {
                let PhysicalSize { width, height } = state.window.inner_size();
                state.canvas.update(&mut state.system, width, height, state.yaw, state.pitch, state.zoom);
                state.window.request_redraw();
            }
            _ => ()
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, canvas: Canvas) {
        if let Application::Initializing(window) = self {
            *self = Application::Initialized(ApplicationState {
                window: { window.request_redraw(); window.clone() },
                last_position: PhysicalPosition::new(0.0, 0.0),
                system: System::new(&canvas.device, "".to_string()),
                rotating: false,
                zoom: 1.0,
                pitch: -30.0,
                yaw: 0.0,
                canvas
            })
        }
    }
}