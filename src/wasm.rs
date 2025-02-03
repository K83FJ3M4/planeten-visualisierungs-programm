#![cfg(target_arch = "wasm32")]
mod entry;
mod canvas;
mod system;

use log::error;
use std::sync::Arc;
use entry::Application;
use wasm_bindgen::prelude::wasm_bindgen;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use web_sys::{window, HtmlCanvasElement};
use winit::platform::web::WindowAttributesExtWebSys;
use winit::window::{Window, WindowAttributes};
use wasm_bindgen::JsCast;
use canvas::Canvas;

#[wasm_bindgen(start)]
pub fn start() {
    console_log::init_with_level(log::Level::Info).ok();
    std::panic::set_hook(Box::new(|panic_info| {
        error!("{panic_info}");
    }));

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut application = Application::new(event_loop.create_proxy());
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut application).unwrap();
}

impl Application {
    fn new(proxy: EventLoopProxy<Canvas>) -> Application {
        let canvas = window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("canvas"))
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok());

        let window_attributes = WindowAttributes::default()
            .with_prevent_default(true)
            .with_focusable(true)
            .with_append(true)
            .with_canvas(canvas);

        Application::Uninitialized(window_attributes, proxy)
    }

    fn init(proxy: EventLoopProxy<Canvas>, window: Arc<Window>) {
        wasm_bindgen_futures::spawn_local(async move {
            let size = window.inner_size();
            let canvas = Canvas::new(window, size).await;
            proxy.send_event(canvas).ok();
        });
    }
}

#[wasm_bindgen]
pub fn handle_file_conent(content: String) {
    *entry::CONTENT.lock().unwrap() = Some(content);
}