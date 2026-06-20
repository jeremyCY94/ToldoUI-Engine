use std::num::NonZero;
use std::rc::Rc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowId};
use winit::dpi::LogicalSize;

use crate::core::app::App;
use crate::core::input;

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        let wa = Window::default_attributes()
            .with_title(&self.default_title)
            .with_inner_size(LogicalSize::new(self.width, self.height));
        let window = el.create_window(wa).unwrap();
        let window = Rc::new(window);

        self.ctx = Some(softbuffer::Context::new(window.clone()).unwrap());
        self.surface = Some(softbuffer::Surface::new(self.ctx.as_ref().unwrap(), window.clone()).unwrap());
        self.window = Some(window.clone());

        let size = window.inner_size();
        self.surface.as_mut().unwrap().resize(NonZero::new(size.width).unwrap(), NonZero::new(size.height).unwrap()).unwrap();
        
        // Iniciar en estado loading y con dom en None para pintar el Skeleton en el primer frame
        self.loading = true;
        self.dom = None;
        self.hovered_node = None;
        self.dragging_node = None;
        window.request_redraw();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, e: winit::event::WindowEvent) {
        match e {
            winit::event::WindowEvent::CloseRequested => el.exit(),
            winit::event::WindowEvent::RedrawRequested => self.draw(),
            winit::event::WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(surf) = &mut self.surface {
                        if let (Some(nw), Some(nh)) = (NonZero::new(size.width), NonZero::new(size.height)) {
                            surf.resize(nw, nh).ok();
                        }
                    }
                    if let Some(w) = &self.window { w.request_redraw(); }
                }
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                input::handle_keyboard(self, event);
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                input::handle_mouse_wheel(self, delta);
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                input::handle_mouse_input(self, state, button);
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                input::handle_cursor_moved(self, position);
            }
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {

        let mut needs_blink = false;
        if let Some(ref focused_key) = self.form.focused {
            let mut is_select = false;
            if let Some(ref dom) = self.dom {
                if let Some(root) = dom.document_element() {
                    fn check_is_select(node: &std::rc::Rc<toldo_ui_engine::dom::Node>, target_key: &str) -> bool {
                        let key = format!("{:p}", toldo_ui_engine::dom::node_ptr(node));
                        if key == target_key {
                            return node.tag_name() == Some("select");
                        }
                        for child in &node.children {
                            if check_is_select(child, target_key) {
                                return true;
                            }
                        }
                        false
                    }
                    is_select = check_is_select(&root, focused_key);
                }
            }
            if !is_select {
                needs_blink = true;
            }
        }

        if needs_blink {
            let now = Instant::now();
            let next_blink = self.last_caret_toggle + std::time::Duration::from_millis(500);
            if now >= next_blink {
                self.caret_on = !self.caret_on;
                self.last_caret_toggle = now;
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            el.set_control_flow(ControlFlow::WaitUntil(self.last_caret_toggle + std::time::Duration::from_millis(500)));
        } else {
            el.set_control_flow(ControlFlow::Wait);
        }
    }
}
