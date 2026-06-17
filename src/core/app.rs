use std::rc::Rc;
use std::time::Instant;

use winit::window::{Window, CursorIcon};
use raqote::DrawTarget;

use toldo_ui_engine::css;
use toldo_ui_engine::dom::{self, DomTree};
use toldo_ui_engine::style::{self, StyleMap};
use toldo_ui_engine::layout;
use toldo_ui_engine::render;
use toldo_ui_engine::form;

pub(crate) struct App {
    pub(crate) window: Option<Rc<Window>>,
    pub(crate) ctx: Option<softbuffer::Context<Rc<Window>>>,
    pub(crate) surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    pub(crate) dom: Option<DomTree>,
    pub(crate) stylesheet: Option<css::Stylesheet>,
    pub(crate) hovered_node: Option<*const dom::Node>,
    pub(crate) styles: StyleMap,
    pub(crate) layout: layout::LayoutEngine,
    pub(crate) painter: render::Painter,
    pub(crate) form: form::FormState,
    pub(crate) scroll_y: f32,
    pub(crate) mouse_x: f32,
    pub(crate) mouse_y: f32,
    pub(crate) dragging_scrollbar: bool,
    pub(crate) dragging_select: bool,
    pub(crate) dragging_node: Option<*const dom::Node>,
    pub(crate) last_click_time: Instant,
    pub(crate) last_click_pos: (f32, f32),
    pub(crate) click_count: u32,
    pub(crate) caret_on: bool,
    pub(crate) last_caret_toggle: Instant,
    pub(crate) default_title: String,
    pub(crate) current_cursor: CursorIcon,
    pub(crate) initial_html: Option<String>,
    pub(crate) initial_css: Option<String>,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) modifiers: winit::keyboard::ModifiersState,
}

impl App {
    pub(crate) fn with_initial_content(mut self, html: &str, css: &str) -> Self {
        self.initial_html = Some(html.to_string());
        self.initial_css = Some(css.to_string());
        self
    }

    pub(crate) fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub(crate) fn new(default_title: &str) -> Self {
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
            last_caret_toggle: std::time::Instant::now(),
            default_title: default_title.to_string(),
            current_cursor: CursorIcon::Default,
            initial_html: None,
            initial_css: None,
            width: 1024.0,
            height: 768.0,
            modifiers: winit::keyboard::ModifiersState::default(),
        }
    }

    pub(crate) fn focus_node(&mut self, key: Option<String>) {
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

    pub(crate) fn load(&mut self, html: &str, css: &str) {
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

    pub(crate) fn update_title(&self) {
        if let Some(ref window) = self.window {
            let title = self.dom.as_ref()
                .and_then(|d| d.title())
                .unwrap_or_else(|| self.default_title.clone());
            window.set_title(&title);
        }
    }

    pub(crate) fn update_cursor_icon(&mut self) {
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

    pub(crate) fn update_hover(&mut self) -> bool {
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

    pub(crate) fn draw(&mut self) {
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
                self.painter.paint(&mut dt, &self.styles, &self.layout, &self.form, root, self.scroll_y, content_h, w as f32, h as f32, self.caret_on, self.mouse_x, self.mouse_y);
            }
        }

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

    pub(crate) fn hit_test(&self, mx: f32, my: f32) -> Option<(std::rc::Rc<dom::Node>, &'static str)> {
        let dom = self.dom.as_ref()?;
        let root = dom.document_element()?;
        self.hit_test_node(&root, mx, my, 0.0, 0.0)
    }

    pub(crate) fn hit_test_node(&self, node: &std::rc::Rc<dom::Node>, mx: f32, my: f32, px: f32, py: f32) -> Option<(std::rc::Rc<dom::Node>, &'static str)> {
        let ptr = dom::node_ptr(node);
        match &node.node_type {
            dom::NodeType::Element(_) => {
                if let Some(lr) = self.layout.get(ptr) {
                    let x = px + lr.location.x;
                    let y = py + lr.location.y;
                    let w = lr.size.width;
                    let h = lr.size.height;

                    if mx >= x && mx < x + w && my >= y && my < y + h || node.tag_name() == Some("html") {
                        for child in &node.children {
                            if let Some(hit) = self.hit_test_node(child, mx, my, x, y) {
                                return Some(hit);
                            }
                        }

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

    pub(crate) fn is_focused_textarea(&self) -> bool {
        let focused_key = match &self.form.focused {
            Some(k) => k,
            None => return false,
        };
        
        fn walk(node: &Rc<dom::Node>, focused_key: &str) -> Option<bool> {
            let key = format!("{:p}", dom::node_ptr(node));
            if key == focused_key {
                return Some(node.tag_name() == Some("textarea"));
            }
            for child in &node.children {
                if let Some(res) = walk(child, focused_key) {
                    return Some(res);
                }
            }
            None
        }

        if let Some(ref dom) = self.dom {
            if let Some(root) = dom.document_element() {
                return walk(&root, focused_key).unwrap_or(false);
            }
        }
        false
    }

    pub(crate) fn scroll_to_caret(&mut self, id: &str) {
        let node_ptr = match self.dom.as_ref() {
            Some(dom) => {
                fn find_node(node: &Rc<dom::Node>, target_key: &str) -> Option<Rc<dom::Node>> {
                    let key = format!("{:p}", dom::node_ptr(node));
                    if key == target_key {
                        return Some(node.clone());
                    }
                    for child in &node.children {
                        if let Some(n) = find_node(child, target_key) {
                            return Some(n);
                        }
                    }
                    None
                }
                let root = match dom.document_element() {
                    Some(r) => r,
                    None => return,
                };
                match find_node(&root, id) {
                    Some(n) => dom::node_ptr(&n),
                    None => return,
                }
            }
            None => return,
        };

        let style = match self.styles.get(&node_ptr) {
            Some(s) => s,
            None => return,
        };

        let lr = match self.layout.get(node_ptr) {
            Some(r) => r,
            None => return,
        };

        let val = self.form.get_value(id);
        let pos = self.form.cursor(id);

        let padding_left = match style.padding_left { style::Length::Px(v) => v, _ => 0.0 };
        let padding_right = match style.padding_right { style::Length::Px(v) => v, _ => 0.0 };
        let border_left = style.border.left.width;
        let border_right = style.border.right.width;
        
        let max_w = lr.size.width - padding_left - padding_right - border_left - border_right - 2.0;
        if max_w <= 0.0 { return; }

        let tw = toldo_ui_engine::render::painter::x_at_index(style, val, pos);
        let mut scroll_x = self.form.get_scroll_x(id);

        if tw - scroll_x < 0.0 {
            scroll_x = tw;
        } else if tw - scroll_x > max_w {
            scroll_x = tw - max_w;
        }

        self.form.set_scroll_x(id, scroll_x.max(0.0));
    }
}

pub(crate) fn populate_form(dom: &DomTree, form: &mut form::FormState) {
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
            "select" => {
                let mut selected_option_key = None;
                let mut selected_val = None;
                for child in &node.children {
                    if child.tag_name() == Some("option") {
                        let opt_key = format!("{:p}", dom::node_ptr(child));
                        if child.get_attribute("selected").is_some() {
                            selected_option_key = Some(opt_key);
                            selected_val = Some(child.children_text().trim().to_string());
                        }
                    }
                }
                if selected_val.is_none() {
                    for child in &node.children {
                        if child.tag_name() == Some("option") {
                            let opt_key = format!("{:p}", dom::node_ptr(child));
                            selected_option_key = Some(opt_key);
                            selected_val = Some(child.children_text().trim().to_string());
                            break;
                        }
                    }
                }
                if let Some(val) = selected_val {
                    form.set_value(&key, val);
                }
                if let Some(opt_key) = selected_option_key {
                    form.checked.insert(opt_key, true);
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

pub(crate) fn get_node_abs_pos(
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
