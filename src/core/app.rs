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
use toldo_ui_engine::render::overlay::{ModalState, ModalType};

pub struct EventListener {
    pub selector: String,
    pub callback: Box<dyn FnMut(&mut App, &Rc<dom::Node>)>,
}

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
    pub(crate) last_dropdown_hover: Option<usize>,
    pub(crate) last_layout_width: u32,
    pub(crate) last_layout_height: u32,
    pub(crate) layout_dirty: bool,
    pub(crate) draw_target: Option<DrawTarget>,
    pub(crate) loading: bool,
    pub(crate) loading_spinner_angle: f32,
    pub(crate) modal: Option<ModalState>,
    pub(crate) click_listeners: Vec<EventListener>,
    pub(crate) deferred_load: Option<(std::time::Instant, String, String)>,
    pub(crate) deferred_action: Option<(std::time::Instant, String)>,
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
            last_dropdown_hover: None,
            last_layout_width: 0,
            last_layout_height: 0,
            layout_dirty: true,
            draw_target: None,
            loading: true,
            loading_spinner_angle: 0.0,
            modal: None,
            click_listeners: Vec::new(),
            deferred_load: None,
            deferred_action: None,
        }
    }

    pub fn rquery(&mut self, selector: &str) -> RQuery<'_> {
        let nodes = self.select_nodes(selector);
        RQuery {
            app: self,
            selector: selector.to_string(),
            nodes,
        }
    }

    pub(crate) fn select_nodes(&self, selector: &str) -> Vec<Rc<dom::Node>> {
        let mut results = Vec::new();
        if let Some(ref dom) = self.dom {
            for node in dom.iter() {
                if matches_selector(&node, selector) {
                    results.push(node.clone());
                }
            }
        }
        results
    }

    pub(crate) fn focus_node(&mut self, key: Option<String>) {
        if self.form.focused != key {
            self.form.focus(key);
            self.resolve_styles();
        }
    }

    pub fn resolve_styles(&mut self) {
        if let Some(ref dom) = self.dom {
            if let Some(root) = dom.document_element() {
                if let Some(ref ss) = self.stylesheet {
                    self.styles = style::resolve_styles(ss, root, self.hovered_node, self.form.focused.as_deref());
                    self.layout_dirty = true;
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
        self.scroll_y = 0.0;
        self.layout_dirty = true;
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
                        let new_styles = style::resolve_styles(ss, root, self.hovered_node, self.form.focused.as_deref());
                        if new_styles != self.styles {
                            self.styles = new_styles;
                            self.layout_dirty = true;
                            return true;
                        }
                    }
                }
            }
        }
        false
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
                if self.layout_dirty || w != self.last_layout_width || h != self.last_layout_height {
                    self.layout.layout(&self.styles, root.clone(), w as f32, h as f32);
                    self.last_layout_width = w;
                    self.last_layout_height = h;
                    self.layout_dirty = false;
                }
                content_h = self.layout.content_height(&root);
                let max_scroll = (content_h - h as f32).max(0.0);

                self.scroll_y = self.scroll_y.clamp(0.0, max_scroll);
            }
        }

        // Comprobar si hay una carga de HTML diferida
        if let Some((time, html, css)) = self.deferred_load.clone() {
            if std::time::Instant::now() >= time {
                self.deferred_load = None;
                self.load(&html, &css);
            }
        }

        // Comprobar si hay una acción diferida
        if let Some((time, action)) = self.deferred_action.clone() {
            if std::time::Instant::now() >= time {
                self.deferred_action = None;
                if action == "submit_success" {
                    if let (Some(html), Some(css)) = (self.initial_html.clone(), self.initial_css.clone()) {
                        self.load(&html, &css);
                    }
                    self.modal = Some(ModalState {
                        title: "Formulario Enviado".to_string(),
                        message: "¡Los datos se han procesado correctamente y la operación fue un éxito!".to_string(),
                        modal_type: ModalType::Alert,
                        action: "submit_success".to_string(),
                    });
                    self.layout_dirty = true;
                }
            }
        }

        if self.loading {
            self.loading_spinner_angle = (self.loading_spinner_angle + 0.15) % std::f32::consts::TAU;
        }

        if self.draw_target.is_none()
            || self.draw_target.as_ref().unwrap().width() != w as i32
            || self.draw_target.as_ref().unwrap().height() != h as i32
        {
            self.draw_target = Some(DrawTarget::new(w as i32, h as i32));
        }

        let dt = self.draw_target.as_mut().unwrap();
        dt.clear(raqote::SolidSource::from_unpremultiplied_argb(255, 255, 255, 255));

        // Resolver la raíz a un documento vacío si dom es None para llamar siempre a painter.paint()
        let root = self.dom.as_ref()
            .and_then(|d| d.document_element())
            .unwrap_or_else(|| dom::Node::new_document());

        self.painter.paint(
            dt,
            &self.styles,
            &self.layout,
            &self.form,
            root,
            self.scroll_y,
            content_h,
            w as f32,
            h as f32,
            self.caret_on,
            self.mouse_x,
            self.mouse_y,
            self.dragging_scrollbar,
            self.loading,
            self.loading_spinner_angle,
            &self.modal,
        );

        let data = dt.get_data();
        if let Some(ref mut surf) = self.surface {
            let mut buf = surf.buffer_mut().unwrap();
            let b = buf.as_mut();
            let min_len = data.len().min(b.len());
            b[..min_len].copy_from_slice(&data[..min_len]);
            if b.len() > min_len {
                b[min_len..].fill(0xFFFFFFFF);
            }
            buf.present().unwrap();
        }

        // Desactivar el loading automáticamente una vez que el DOM está listo y se ha dibujado
        if self.loading && self.dom.is_some() && self.deferred_load.is_none() && self.deferred_action.is_none() {
            self.loading = false;
            self.layout_dirty = true;
            if let Some(ref w) = self.window {
                w.request_redraw();
            }
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

    pub(crate) fn get_focusable_nodes(&self) -> Vec<String> {
        let mut focusables = Vec::new();
        if let Some(ref dom) = self.dom {
            if let Some(root) = dom.document_element() {
                fn walk(node: &Rc<dom::Node>, list: &mut Vec<String>) {
                    let tag = node.tag_name().unwrap_or("");
                    let is_focusable = match tag {
                        "input" | "textarea" | "select" | "button" | "a" => true,
                        _ => false,
                    };
                    if is_focusable {
                        list.push(format!("{:p}", dom::node_ptr(node)));
                    }
                    for child in &node.children {
                        walk(child, list);
                    }
                }
                walk(&root, &mut focusables);
            }
        }
        focusables
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
                if node.get_attribute("checked").is_some() {
                    form.checked.insert(key.clone(), true);
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
                if node.get_attribute("selected").is_some() {
                    form.checked.insert(key.clone(), true);
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

pub fn matches_selector(node: &Rc<dom::Node>, selector: &str) -> bool {
    if !node.is_element() {
        return false;
    }
    if selector.starts_with('#') {
        let id_val = &selector[1..];
        node.id() == Some(id_val)
    } else if selector.starts_with('.') {
        let class_val = &selector[1..];
        node.has_class(class_val)
    } else {
        node.tag_name().map(|t| t.to_lowercase()) == Some(selector.to_lowercase())
    }
}

pub struct RQuery<'a> {
    pub app: &'a mut App,
    pub selector: String,
    pub nodes: Vec<Rc<dom::Node>>,
}

impl<'a> RQuery<'a> {
    pub fn on_click<F>(self, mut handler: F) -> Self
    where
        F: FnMut(&mut App, &Rc<dom::Node>) + 'static,
    {
        self.app.click_listeners.push(EventListener {
            selector: self.selector.clone(),
            callback: Box::new(move |app, node| {
                handler(app, node);
            }),
        });
        self
    }

    #[allow(dead_code)]
    pub fn text(&self) -> String {
        self.nodes.first().map(|n| n.children_text()).unwrap_or_default()
    }

    pub fn set_text(self, text: &str) -> Self {
        for node in &self.nodes {
            let mut_ptr = Rc::as_ptr(node) as *mut dom::Node;
            unsafe {
                (*mut_ptr).children.clear();
                let text_node = dom::Node::new_text(text.to_string());
                text_node.set_parent(Rc::as_ptr(node));
                (*mut_ptr).children.push(text_node);
            }
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn attr(&self, name: &str) -> Option<String> {
        self.nodes.first().and_then(|n| n.get_attribute(name).map(|s| s.to_string()))
    }

    pub fn set_attr(self, name: &str, value: &str) -> Self {
        for node in &self.nodes {
            let mut_ptr = Rc::as_ptr(node) as *mut dom::Node;
            unsafe {
                if let dom::NodeType::Element(ref mut data) = (*mut_ptr).node_type {
                    if value.is_empty() {
                        data.attributes.remove(name);
                    } else {
                        data.attributes.insert(name.to_string(), value.to_string());
                    }
                }
            }
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn set_html(self, html: &str) -> Self {
        for node in &self.nodes {
            let mut_ptr = Rc::as_ptr(node) as *mut dom::Node;
            unsafe {
                (*mut_ptr).children.clear();
            }
            let mut parser = toldo_ui_engine::dom::parser::HtmlParser::new(html);
            parser.parse_children(node, true);
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn add_class(self, class_name: &str) -> Self {
        for node in &self.nodes {
            let mut_ptr = Rc::as_ptr(node) as *mut dom::Node;
            unsafe {
                if let dom::NodeType::Element(ref mut data) = (*mut_ptr).node_type {
                    let existing = data.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
                    if !existing.split_whitespace().any(|c| c == class_name) {
                        let new_class = if existing.is_empty() {
                            class_name.to_string()
                        } else {
                            format!("{} {}", existing, class_name)
                        };
                        data.attributes.insert("class".to_string(), new_class);
                    }
                }
            }
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn remove_class(self, class_name: &str) -> Self {
        for node in &self.nodes {
            let mut_ptr = Rc::as_ptr(node) as *mut dom::Node;
            unsafe {
                if let dom::NodeType::Element(ref mut data) = (*mut_ptr).node_type {
                    if let Some(existing) = data.attributes.get("class") {
                        let filtered: Vec<&str> = existing.split_whitespace().filter(|&c| c != class_name).collect();
                        if filtered.is_empty() {
                            data.attributes.remove("class");
                        } else {
                            data.attributes.insert("class".to_string(), filtered.join(" "));
                        }
                    }
                }
            }
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn toggle_class(self, class_name: &str) -> Self {
        for node in &self.nodes {
            let has_it = node.has_class(class_name);
            let mut_ptr = Rc::as_ptr(node) as *mut dom::Node;
            unsafe {
                if let dom::NodeType::Element(ref mut data) = (*mut_ptr).node_type {
                    let existing = data.attributes.get("class").map(|s| s.as_str()).unwrap_or("");
                    if has_it {
                        let filtered: Vec<&str> = existing.split_whitespace().filter(|&c| c != class_name).collect();
                        if filtered.is_empty() {
                            data.attributes.remove("class");
                        } else {
                            data.attributes.insert("class".to_string(), filtered.join(" "));
                        }
                    } else {
                        let new_class = if existing.is_empty() {
                            class_name.to_string()
                        } else {
                            format!("{} {}", existing, class_name)
                        };
                        data.attributes.insert("class".to_string(), new_class);
                    }
                }
            }
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn has_class(&self, class_name: &str) -> bool {
        self.nodes.first().map(|n| n.has_class(class_name)).unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn css(self, property: &str, value: &str) -> Self {
        for node in &self.nodes {
            let mut_ptr = Rc::as_ptr(node) as *mut dom::Node;
            unsafe {
                if let dom::NodeType::Element(ref mut data) = (*mut_ptr).node_type {
                    let existing_style = data.attributes.get("style").map(|s| s.as_str()).unwrap_or("");
                    let mut declarations = std::collections::HashMap::new();
                    for decl in existing_style.split(';') {
                        let parts: Vec<&str> = decl.split(':').collect();
                        if parts.len() == 2 {
                            declarations.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                        }
                    }
                    if value.is_empty() {
                        declarations.remove(property);
                    } else {
                        declarations.insert(property.to_string(), value.to_string());
                    }
                    let new_style = declarations.iter()
                        .map(|(k, v)| format!("{}: {};", k, v))
                        .collect::<Vec<String>>()
                        .join(" ");
                    if new_style.is_empty() {
                        data.attributes.remove("style");
                    } else {
                        data.attributes.insert("style".to_string(), new_style);
                    }
                }
            }
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn hide(self) -> Self {
        self.css("display", "none")
    }

    #[allow(dead_code)]
    pub fn show(self) -> Self {
        self.css("display", "")
    }

    #[allow(dead_code)]
    pub fn val(&self) -> String {
        if let Some(node) = self.nodes.first() {
            let key = format!("{:p}", Rc::as_ptr(node));
            self.app.form.get_value(&key).to_string()
        } else {
            String::new()
        }
    }

    #[allow(dead_code)]
    pub fn set_val(self, value: &str) -> Self {
        for node in &self.nodes {
            let key = format!("{:p}", Rc::as_ptr(node));
            self.app.form.set_value(&key, value.to_string());
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn is_checked(&self) -> bool {
        if let Some(node) = self.nodes.first() {
            let key = format!("{:p}", Rc::as_ptr(node));
            self.app.form.is_checked(&key)
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn set_checked(self, checked: bool) -> Self {
        for node in &self.nodes {
            let key = format!("{:p}", Rc::as_ptr(node));
            self.app.form.checked.insert(key, checked);
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn append(self, html: &str) -> Self {
        for node in &self.nodes {
            let mut parser = toldo_ui_engine::dom::parser::HtmlParser::new(html);
            parser.parse_children(node, true);
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn prepend(self, html: &str) -> Self {
        for node in &self.nodes {
            let temp_parent = dom::Node::new_document();
            let mut parser = toldo_ui_engine::dom::parser::HtmlParser::new(html);
            parser.parse_children(&temp_parent, true);
            let new_children = temp_parent.children.clone();
            if !new_children.is_empty() {
                let mut_ptr = Rc::as_ptr(node) as *mut dom::Node;
                unsafe {
                    for child in &new_children {
                        child.set_parent(Rc::as_ptr(node));
                    }
                    let existing = std::mem::take(&mut (*mut_ptr).children);
                    (*mut_ptr).children.extend(new_children);
                    (*mut_ptr).children.extend(existing);
                }
            }
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn empty(self) -> Self {
        for node in &self.nodes {
            let mut_ptr = Rc::as_ptr(node) as *mut dom::Node;
            unsafe {
                (*mut_ptr).children.clear();
            }
        }
        self.app.resolve_styles();
        self
    }

    #[allow(dead_code)]
    pub fn remove(self) -> Self {
        for node in &self.nodes {
            if let Some(parent) = node.get_parent() {
                let parent_mut = parent as *const dom::Node as *mut dom::Node;
                unsafe {
                    (*parent_mut).children.retain(|c| Rc::as_ptr(c) != Rc::as_ptr(node));
                }
            }
        }
        self.app.resolve_styles();
        self
    }
}
