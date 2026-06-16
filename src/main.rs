mod core;

use crate::core::app::App;
use winit::event_loop::EventLoop;

fn main() {
    let el = EventLoop::new().unwrap();
    let html = include_str!("../examples/simple.html");
    let css = include_str!("../examples/simple.css");
    let mut app = App::new("ToldoUI-Engine")
        .with_initial_content(html, css)
        .with_size(1024.0, 768.0);
    el.run_app(&mut app).unwrap();
}
