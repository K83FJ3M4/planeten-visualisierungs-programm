#![cfg_attr(target_arch = "wasm32", no_main)]
#![cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
use log::{error, LevelFilter};
use simplelog::{Config, SimpleLogger};
use winit::dpi::PhysicalSize;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::window::{Window, WindowAttributes};
use pollster::FutureExt;
use entry::Application;
use canvas::Canvas;

mod entry;
mod canvas;
mod system;

fn main() {
    SimpleLogger::init(LevelFilter::Info, Config::default()).ok();
    std::panic::set_hook(Box::new(|panic_info| {
        error!("{panic_info}");
    }));

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut application = Application::new(event_loop.create_proxy());
    event_loop.run_app(&mut application).unwrap();
}

impl Application {
    fn new(proxy: EventLoopProxy<Canvas>) -> Application {
        let window_attributes = WindowAttributes::default()
            .with_title("WebGPU User Interface")
            .with_inner_size(PhysicalSize::new(800, 600));

        Application::Uninitialized(window_attributes, proxy)
    }

    fn init(proxy: EventLoopProxy<Canvas>, window: Arc<Window>) {
        let size = window.inner_size();
        let context = Canvas::new(window, size).block_on();
        proxy.send_event(context).ok();
    }
}