use std::num::NonZero;
use std::rc::Rc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId, CursorIcon};

use raqote::*;

use toldo_ui_engine::css;
use toldo_ui_engine::dom::{self, DomTree};
use toldo_ui_engine::style::{self, StyleMap};
use toldo_ui_engine::layout;
use toldo_ui_engine::render;
use toldo_ui_engine::form;

const W: u32 = 1024;
const H: u32 = 768;

struct App {
    window: Option<Rc<Window>>,
    ctx: Option<softbuffer::Context<Rc<Window>>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    dom: Option<DomTree>,
    stylesheet: Option<css::Stylesheet>,
    hovered_node: Option<*const dom::Node>,
    styles: StyleMap,
    layout: layout::LayoutEngine,
    painter: render::Painter,
    form: form::FormState,
    scroll_y: f32,
    mouse_x: f32,
    mouse_y: f32,
    dragging_scrollbar: bool,
    dragging_select: bool,
    dragging_node: Option<*const dom::Node>,
    last_click_time: Instant,
    last_click_pos: (f32, f32),
    click_count: u32,
    caret_on: bool,
    default_title: String,
    current_cursor: CursorIcon,
}

impl App {
    fn new(default_title: &str) -> Self {
        App {
            window: None, ctx: None, surface: None,
            dom: None, stylesheet: None, hovered_node: None,
            styles: StyleMap::new(),
            layout: layout::LayoutEngine::new(),
            painter: render::Painter::new(),
            form: form::FormState::new(),
            scroll_y: 0.0,
            mouse_x: 0.0, mouse_y: 0.0,
            dragging_scrollbar: false,
            dragging_select: false,
            dragging_node: None,
            last_click_time: std::time::Instant::now(),
            last_click_pos: (0.0, 0.0),
            click_count: 0,
            caret_on: true,
            default_title: default_title.to_string(),
            current_cursor: CursorIcon::Default,
        }
    }

    fn focus_node(&mut self, key: Option<String>) {
        if self.form.focused != key {
            self.form.focus(key);
            if let Some(ref dom) = self.dom {
                if let Some(root) = dom.document_element() {
                    if let Some(ref ss) = self.stylesheet {
                        self.styles = style::resolve_styles(ss, root, self.hovered_node, self.form.focused.as_deref());
                    }
                }
            }
        }
    }

    fn load(&mut self, html: &str, css: &str) {
        let dom = DomTree::parse_html(html);
        let ss = css::Stylesheet::parse(css);
        if let Some(root) = dom.document_element() {
            self.styles = style::resolve_styles(&ss, root, self.hovered_node, self.form.focused.as_deref());
        }
        self.stylesheet = Some(ss);
        self.form = form::FormState::new();
        populate_form(&dom, &mut self.form);
        self.dom = Some(dom);
        self.update_title();
    }

    fn update_title(&self) {
        if let Some(ref window) = self.window {
            let title = self.dom.as_ref()
                .and_then(|d| d.title())
                .unwrap_or_else(|| self.default_title.clone());
            window.set_title(&title);
        }
    }

    fn update_cursor_icon(&mut self) {
        let hit = self.hit_test(self.mouse_x, self.mouse_y + self.scroll_y);
        let needed_cursor = match hit {
            Some((node, form_type)) => {
                let css_cursor = self.styles.get(&dom::node_ptr(&node))
                    .map(|style| style.cursor)
                    .unwrap_or(style::Cursor::Auto);

                match css_cursor {
                    style::Cursor::Default => CursorIcon::Default,
                    style::Cursor::Pointer => CursorIcon::Pointer,
                    style::Cursor::Text => CursorIcon::Text,
                    style::Cursor::Wait => CursorIcon::Wait,
                    style::Cursor::Help => CursorIcon::Help,
                    style::Cursor::NotAllowed => CursorIcon::NotAllowed,
                    style::Cursor::Progress => CursorIcon::Progress,
                    style::Cursor::Grab => CursorIcon::Grab,
                    style::Cursor::Grabbing => CursorIcon::Grabbing,
                    style::Cursor::Move => CursorIcon::Move,
                    style::Cursor::ZoomIn => CursorIcon::ZoomIn,
                    style::Cursor::ZoomOut => CursorIcon::ZoomOut,
                    style::Cursor::Auto => match form_type {
                        "text" | "textarea" => CursorIcon::Text,
                        "button" | "select" | "link" | "checkbox" | "radio" => CursorIcon::Pointer,
                        _ => CursorIcon::Default,
                    }
                }
            }
            None => {
                let win = match &self.window { Some(w) => w.clone(), None => return };
                let size = win.inner_size();
                let ww = size.width.max(100) as f32;
                let wh = size.height.max(100) as f32;
                let sb_w = 10.0;
                let sb_x = ww - sb_w - 2.0;
                if self.mouse_x >= sb_x && self.mouse_x < sb_x + sb_w && self.mouse_y >= 2.0 && self.mouse_y < wh - 2.0 {
                    CursorIcon::Pointer
                } else {
                    CursorIcon::Default
                }
            }
        };

        if self.current_cursor != needed_cursor {
            if let Some(ref window) = self.window {
                window.set_cursor(needed_cursor);
            }
            self.current_cursor = needed_cursor;
        }
    }

    fn update_hover(&mut self) -> bool {
        let hovered = self.hit_test(self.mouse_x, self.mouse_y + self.scroll_y)
            .map(|(node, _)| dom::node_ptr(&node));

        if self.hovered_node != hovered {
            self.hovered_node = hovered;
            if let Some(ref dom) = self.dom {
                if let Some(root) = dom.document_element() {
                    if let Some(ref ss) = self.stylesheet {
                        self.styles = style::resolve_styles(ss, root, self.hovered_node, self.form.focused.as_deref());
                    }
                }
            }
            true
        } else {
            false
        }
    }

    fn draw(&mut self) {
        let window = match &self.window { Some(w) => w.clone(), None => return };
        let _ctx = match &self.ctx { Some(c) => c.clone(), None => return };

        let size = window.inner_size();
        let w = size.width.max(100);
        let h = size.height.max(100);

        let mut content_h = 0.0;
        if let Some(ref dom) = self.dom {
            if let Some(root) = dom.document_element() {
                self.layout.layout(&self.styles, root.clone(), w as f32, h as f32);
                content_h = self.layout.content_height(&root);
                let max_scroll = (content_h - h as f32).max(0.0);
                self.scroll_y = self.scroll_y.min(max_scroll);
            }
        }

        let mut dt = DrawTarget::new(w as i32, h as i32);
        if let Some(ref dom) = self.dom {
            if let Some(root) = dom.document_element() {
                self.painter.paint(&mut dt, &self.styles, &self.layout, &self.form, root, self.scroll_y, content_h, w as f32, h as f32, self.caret_on);
            }
        }

        self.caret_on = !self.caret_on;

        let data = dt.get_data();
        let count = (w * h) as usize;

        if let Some(ref mut surf) = self.surface {
            let mut buf = surf.buffer_mut().unwrap();
            let b = buf.as_mut();
            for i in 0..count.min(b.len()) {
                let p = if i < data.len() { data[i] } else { 0xFFFFFFFF };
                let a = (p >> 24) & 0xFF;
                let r = (p >> 16) & 0xFF;
                let g = (p >> 8) & 0xFF;
                let bl = p & 0xFF;
                b[i] = bl | (g << 8) | (r << 16) | (a << 24);
            }
            buf.present().unwrap();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        let wa = Window::default_attributes()
            .with_title(&self.default_title)
            .with_inner_size(LogicalSize::new(W as f64, H as f64));
        let window = el.create_window(wa).unwrap();
        let window = Rc::new(window);

        self.ctx = Some(softbuffer::Context::new(window.clone()).unwrap());
        self.surface = Some(softbuffer::Surface::new(self.ctx.as_ref().unwrap(), window.clone()).unwrap());
        self.window = Some(window.clone());

        let size = window.inner_size();
        self.surface.as_mut().unwrap().resize(NonZero::new(size.width).unwrap(), NonZero::new(size.height).unwrap()).unwrap();
        self.load(include_str!("../examples/simple.html"), include_str!("../examples/simple.css"));
        window.request_redraw();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, e: WindowEvent) {
        match e {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(surf) = &mut self.surface {
                        if let (Some(nw), Some(nh)) = (NonZero::new(size.width), NonZero::new(size.height)) {
                            surf.resize(nw, nh).ok();
                        }
                    }
                    if let Some(w) = &self.window { w.request_redraw(); }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let Some(ref focused) = self.form.focused.clone() {
                        match &event.logical_key {
                            Key::Named(NamedKey::Backspace) => {
                                if !delete_selected_text(&mut self.form, focused) {
                                    let len = self.form.get_value(focused).chars().count();
                                    if len > 0 {
                                        let pos = self.form.cursor(focused).min(len);
                                        if pos > 0 {
                                            let mut chars: Vec<char> = self.form.get_value(focused).chars().collect();
                                            chars.remove(pos - 1);
                                            let new_val: String = chars.into_iter().collect();
                                            self.form.set_value(focused, new_val);
                                            self.form.set_cursor(focused, pos - 1);
                                        }
                                    }
                                }
                                if let Some(w) = &self.window { w.request_redraw(); }
                            }
                            Key::Named(NamedKey::Enter) => {
                                delete_selected_text(&mut self.form, focused);
                                let len = self.form.get_value(focused).chars().count();
                                let pos = self.form.cursor(focused).min(len);
                                let mut chars: Vec<char> = self.form.get_value(focused).chars().collect();
                                chars.insert(pos, '\n');
                                let new_val: String = chars.into_iter().collect();
                                self.form.set_value(focused, new_val);
                                self.form.set_cursor(focused, pos + 1);
                                if let Some(w) = &self.window { w.request_redraw(); }
                            }
                            Key::Named(NamedKey::Escape) => { self.focus_node(None); if let Some(w) = &self.window { w.request_redraw(); } }
                            Key::Character(c) if c.len() == 1 && c.as_bytes()[0] >= 32 => {
                                delete_selected_text(&mut self.form, focused);
                                let len = self.form.get_value(focused).chars().count();
                                let pos = self.form.cursor(focused).min(len);
                                let mut chars: Vec<char> = self.form.get_value(focused).chars().collect();
                                chars.insert(pos, c.chars().next().unwrap());
                                let new_val: String = chars.into_iter().collect();
                                self.form.set_value(focused, new_val);
                                self.form.set_cursor(focused, pos + 1);
                                if let Some(w) = &self.window { w.request_redraw(); }
                            }
                            _ => {}
                        }
                    } else {
                        match &event.logical_key {
                            Key::Named(NamedKey::ArrowUp) => { self.scroll_y = (self.scroll_y - 30.0).max(0.0); if let Some(w) = &self.window { w.request_redraw(); } }
                            Key::Named(NamedKey::ArrowDown) => { self.scroll_y += 30.0; if let Some(w) = &self.window { w.request_redraw(); } }
                            Key::Named(NamedKey::PageUp) => { self.scroll_y = (self.scroll_y - 300.0).max(0.0); if let Some(w) = &self.window { w.request_redraw(); } }
                            Key::Named(NamedKey::PageDown) => { self.scroll_y += 300.0; if let Some(w) = &self.window { w.request_redraw(); } }
                            Key::Character(c) if c == "r" || c == "R" => {
                                self.load(include_str!("../examples/simple.html"), include_str!("../examples/simple.css"));
                                if let Some(w) = &self.window { w.request_redraw(); }
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), .. } => {
                self.scroll_y = (self.scroll_y - y * 20.0).max(0.0);
                self.dragging_scrollbar = false;
                self.update_cursor_icon();
                self.update_hover();
                if let Some(w) = &self.window { w.request_redraw(); }
            }
            WindowEvent::MouseWheel { delta: MouseScrollDelta::PixelDelta(pos), .. } => {
                self.scroll_y = (self.scroll_y - pos.y as f32).max(0.0);
                self.dragging_scrollbar = false;
                self.update_cursor_icon();
                self.update_hover();
                if let Some(w) = &self.window { w.request_redraw(); }
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                let win = match &self.window { Some(w) => w.clone(), None => return };
                let size = win.inner_size();
                let ww = size.width.max(100) as f32;
                let wh = size.height.max(100) as f32;
                let ch = self.dom.as_ref().and_then(|d| d.document_element()).map(|r| self.layout.content_height(&r)).unwrap_or(0.0);
                let sb_w = 10.0;
                let sb_x = ww - sb_w - 2.0;
                let track_h = wh - 4.0;
                let thumb_h = (wh / ch * track_h).max(20.0);
                let max_scroll = (ch - wh).max(0.0);
 
                match state {
                    ElementState::Pressed => {
                        // Click on form element
                        let now = std::time::Instant::now();
                        let dt = now - self.last_click_time;
                        if dt.as_millis() < 400 && (self.mouse_x - self.last_click_pos.0).abs() < 8.0 && (self.mouse_y - self.last_click_pos.1).abs() < 8.0 {
                            self.click_count += 1;
                        } else { self.click_count = 1; }
                        self.last_click_time = now;
                        self.last_click_pos = (self.mouse_x, self.mouse_y);
 
                        if let Some((node, form_type)) = self.hit_test(self.mouse_x, self.mouse_y + self.scroll_y) {
                            let key = format!("{:p}", dom::node_ptr(&node));
                            match form_type {
                                "checkbox" => { self.form.toggle(&key); self.focus_node(None); self.click_count = 0; }
                                "radio" => { self.form.toggle(&key); self.focus_node(None); self.click_count = 0; }
                                "text" | "textarea" => {
                                     self.focus_node(Some(key.clone()));
                                     let val = self.form.get_value(&key);
                                     let mut start_idx = val.chars().count();
                                     let node_ptr = dom::node_ptr(&node);
                                     if let Some(root) = self.dom.as_ref().and_then(|d| d.document_element()) {
                                         if let Some((node_x, _)) = get_node_abs_pos(&root, node_ptr, &self.layout, 0.0, 0.0) {
                                             if let Some(style) = self.styles.get(&node_ptr) {
                                                 let padding_left = match style.padding_left { crate::style::Length::Px(v) => v, _ => 0.0 };
                                                 let border_left = style.border.left.width;
                                                 let cx = node_x + padding_left + border_left;
                                                 let target_x = self.mouse_x - cx;
                                                 start_idx = toldo_ui_engine::render::painter::index_at_x(style, val, target_x);
                                             }
                                         }
                                     }
                                     if self.click_count >= 2 {
                                         self.form.select_all(&key);
                                     } else {
                                         self.form.set_cursor(&key, start_idx);
                                         self.form.set_selection(&key, start_idx, start_idx);
                                     }
                                     self.caret_on = true;
                                     self.dragging_select = true;
                                     self.dragging_node = Some(node_ptr);
                                }
                                _ => {
                                    self.focus_node(None);
                                    self.click_count = 0;
                                }
                            }
                            if let Some(w) = &self.window { w.request_redraw(); }
                        } else {
                            self.focus_node(None);
                            self.click_count = 0;
                        }
 
                        // Click on scrollbar thumb → start dragging
                        if self.mouse_y >= 2.0 && self.mouse_y < 2.0 + track_h && self.mouse_x >= sb_x && self.mouse_x < sb_x + sb_w {
                            let thumb_y = if max_scroll > 0.0 { 2.0 + (self.scroll_y / max_scroll) * (track_h - thumb_h) } else { 2.0 };
                            if self.mouse_y >= thumb_y && self.mouse_y < thumb_y + thumb_h {
                                self.dragging_scrollbar = true;
                            } else {
                                // Click track → jump
                                let ratio = ((self.mouse_y - 2.0) - thumb_h * 0.5) / (track_h - thumb_h);
                                self.scroll_y = (ratio * max_scroll).clamp(0.0, max_scroll);
                                if let Some(w) = &self.window { w.request_redraw(); }
                            }
                        }
                    }
                    ElementState::Released => {
                        self.dragging_scrollbar = false;
                        self.dragging_select = false;
                        self.dragging_node = None;
                    }
                }
                self.update_cursor_icon();
                self.update_hover();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_x = position.x as f32;
                self.mouse_y = position.y as f32;
 
                if self.dragging_scrollbar {
                    let win = match &self.window { Some(w) => w.clone(), None => return };
                    let size = win.inner_size();
                    let wh = size.height.max(100) as f32;
                    let ch = self.dom.as_ref().and_then(|d| d.document_element()).map(|r| self.layout.content_height(&r)).unwrap_or(0.0);
                    let max_scroll = (ch - wh).max(0.0);
                    let track_h = wh - 4.0;
                    let thumb_h = (wh / ch * track_h).max(20.0);
                    let ratio = (self.mouse_y - 2.0 - thumb_h * 0.5) / (track_h - thumb_h);
                    self.scroll_y = (ratio * max_scroll).clamp(0.0, max_scroll);
                    if let Some(w) = &self.window { w.request_redraw(); }
                }
 
                if self.dragging_select {
                    if let Some(node_ptr) = self.dragging_node {
                        let key = format!("{:p}", node_ptr);
                        if let Some(root) = self.dom.as_ref().and_then(|d| d.document_element()) {
                            if let Some((node_x, _)) = get_node_abs_pos(&root, node_ptr, &self.layout, 0.0, 0.0) {
                                if let Some(style) = self.styles.get(&node_ptr) {
                                    let padding_left = match style.padding_left { crate::style::Length::Px(v) => v, _ => 0.0 };
                                    let border_left = style.border.left.width;
                                    let cx = node_x + padding_left + border_left;
                                    let val = self.form.get_value(&key);
                                    let target_x = self.mouse_x - cx;
                                    let current_idx = toldo_ui_engine::render::painter::index_at_x(style, val, target_x);
 
                                    if let Some((start_idx, _)) = self.form.get_selection(&key) {
                                        self.form.set_selection(&key, start_idx, current_idx);
                                        self.form.set_cursor(&key, current_idx);
                                        if let Some(w) = &self.window { w.request_redraw(); }
                                    }
                                }
                            }
                        }
                    }
                }
  
                self.update_cursor_icon();
                if self.update_hover() {
                    if let Some(w) = &self.window { w.request_redraw(); }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if self.form.focused.is_some() {
            el.set_control_flow(ControlFlow::Poll);
        } else {
            el.set_control_flow(ControlFlow::Wait);
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: winit::event::DeviceId, _: DeviceEvent) {}
}

fn populate_form(dom: &DomTree, form: &mut form::FormState) {
    fn walk(node: &Rc<dom::Node>, form: &mut form::FormState) {
        let tag = node.tag_name().unwrap_or("");
        let key = format!("{:p}", dom::node_ptr(node));
        match tag {
            "input" => {
                if let Some(val) = node.get_attribute("value") {
                    if !val.is_empty() { form.set_value(&key, val.to_string()); }
                }
                if let Some(chk) = node.get_attribute("checked") {
                    if chk == "checked" { form.checked.insert(key.clone(), true); }
                }
            }
            "textarea" => {
                for child in &node.children {
                    if let dom::NodeType::Text(ref t) = child.node_type {
                        form.set_value(&key, t.clone());
                    }
                }
            }
            "option" => {
                if let Some(sel) = node.get_attribute("selected") {
                    if sel == "selected" { form.checked.insert(key.clone(), true); }
                }
            }
            _ => {}
        }
        for child in &node.children { walk(child, form); }
    }
    if let Some(root) = dom.document_element() { walk(&root, form); }
}

impl App {
    fn hit_test(&self, mx: f32, my: f32) -> Option<(std::rc::Rc<dom::Node>, &'static str)> {
        let dom = self.dom.as_ref()?;
        let root = dom.document_element()?;
        self.hit_test_node(&root, mx, my, 0.0, 0.0)
    }

    fn hit_test_node(&self, node: &std::rc::Rc<dom::Node>, mx: f32, my: f32, px: f32, py: f32) -> Option<(std::rc::Rc<dom::Node>, &'static str)> {
        let ptr = dom::node_ptr(node);
        match &node.node_type {
            dom::NodeType::Element(_) => {
                if let Some(lr) = self.layout.get(ptr) {
                    let x = px + lr.location.x;
                    let y = py + lr.location.y;
                    let w = lr.size.width;
                    let h = lr.size.height;

                    if mx >= x && mx < x + w && my >= y && my < y + h || node.tag_name() == Some("html") {
                        // 1. Probar en los hijos primero
                        for child in &node.children {
                            if let Some(hit) = self.hit_test_node(child, mx, my, x, y) {
                                return Some(hit);
                            }
                        }

                        // 2. Si ningún hijo hace hit, el elemento actual es el hit
                        let itype = node.get_attribute("type").unwrap_or("");
                        let form_type = match node.tag_name().unwrap_or("") {
                            "input" => match itype { "checkbox" => "checkbox", "radio" => "radio", _ => "text" },
                            "button" => "button", "select" => "select", "textarea" => "textarea",
                            "a" => "link",
                            _ => "generic",
                        };
                        return Some((node.clone(), form_type));
                    }
                }
                None
            }
            dom::NodeType::Text(_) | dom::NodeType::Document => {
                for child in &node.children {
                    if let Some(hit) = self.hit_test_node(child, mx, my, px, py) {
                        return Some(hit);
                    }
                }
                None
            }
        }
    }
}

fn delete_selected_text(form: &mut form::FormState, id: &str) -> bool {
    if let Some((start, end)) = form.get_selection(id) {
        if start != end {
            let s_min = start.min(end);
            let s_max = start.max(end);
            let val = form.get_value(id);
            let chars: Vec<char> = val.chars().collect();
            if s_min < chars.len() {
                let mut new_chars = Vec::new();
                for i in 0..chars.len() {
                    if i < s_min || i >= s_max {
                        new_chars.push(chars[i]);
                    }
                }
                let new_val: String = new_chars.into_iter().collect();
                form.set_value(id, new_val);
                form.set_cursor(id, s_min);
            } else if chars.is_empty() {
                form.set_value(id, String::new());
                form.set_cursor(id, 0);
            }
            form.clear_selection(id);
            return true;
        }
    }
    false
}

fn get_node_abs_pos(
    root: &Rc<dom::Node>,
    target_ptr: *const dom::Node,
    layout: &layout::LayoutEngine,
    px: f32,
    py: f32,
) -> Option<(f32, f32)> {
    let ptr = dom::node_ptr(root);
    if ptr == target_ptr {
        if let Some(lr) = layout.get(ptr) {
            return Some((px + lr.location.x, py + lr.location.y));
        }
    }
    if let Some(lr) = layout.get(ptr) {
        let x = px + lr.location.x;
        let y = py + lr.location.y;
        for child in &root.children {
            if let Some(pos) = get_node_abs_pos(child, target_ptr, layout, x, y) {
                return Some(pos);
            }
        }
    }
    None
}

fn main() {
    let el = EventLoop::new().unwrap();
    el.run_app(&mut App::new("ToldoUI-Engine")).unwrap();
}
