use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::{Key, NamedKey};
use winit::dpi::PhysicalPosition;

use toldo_ui_engine::dom;
use toldo_ui_engine::form::actions::{delete_selected_text, get_selected_text, insert_text};
use toldo_ui_engine::render::overlay::{ModalState, ModalType};

use crate::core::app::{App, get_node_abs_pos};

pub(crate) fn handle_keyboard(app: &mut App, event: winit::event::KeyEvent) {
    if app.modal.is_some() {
        return;
    }
    if event.state == ElementState::Pressed {
        // Recarga global con F5
        if event.logical_key == Key::Named(NamedKey::F5) {
            app.loading = true;
            app.dom = None;
            app.hovered_node = None;
            app.dragging_node = None;
            if let Some(w) = &app.window {
                w.request_redraw();
            }
            return;
        }

        if event.logical_key == Key::Named(NamedKey::Tab) {
            let focusables = app.get_focusable_nodes();
            if !focusables.is_empty() {
                let shift_pressed = app.modifiers.shift_key();
                let next_idx = match &app.form.focused {
                    Some(curr_focused) => {
                        if let Some(idx) = focusables.iter().position(|k| k == curr_focused) {
                            if shift_pressed {
                                if idx == 0 {
                                    focusables.len() - 1
                                } else {
                                    idx - 1
                                }
                            } else {
                                (idx + 1) % focusables.len()
                            }
                        } else {
                            if shift_pressed { focusables.len() - 1 } else { 0 }
                        }
                    }
                    None => {
                        if shift_pressed { focusables.len() - 1 } else { 0 }
                    }
                };

                let new_key = focusables[next_idx].clone();
                app.focus_node(Some(new_key.clone()));
                app.caret_on = true;
                app.last_caret_toggle = std::time::Instant::now();

                let val_len = app.form.get_value(&new_key).chars().count();
                app.form.set_cursor(&new_key, val_len);
                app.form.select_all(&new_key);

                if let Some(ref dom) = app.dom {
                    if let Some(root) = dom.document_element() {
                        fn find_node_ptr(node: &std::rc::Rc<toldo_ui_engine::dom::Node>, target_key: &str) -> Option<*const toldo_ui_engine::dom::Node> {
                            let key = format!("{:p}", toldo_ui_engine::dom::node_ptr(node));
                            if key == target_key {
                                return Some(toldo_ui_engine::dom::node_ptr(node));
                            }
                            for child in &node.children {
                                if let Some(n) = find_node_ptr(child, target_key) {
                                    return Some(n);
                                }
                            }
                            None
                        }

                        if let Some(node_ptr) = find_node_ptr(&root, &new_key) {
                            if let Some((_, node_y)) = get_node_abs_pos(&root, node_ptr, &app.layout, 0.0, 0.0) {
                                if let Some(lr) = app.layout.get(node_ptr) {
                                    let node_h = lr.size.height;
                                    if let Some(ref win) = app.window {
                                        let wh = win.inner_size().height.max(100) as f32;
                                        let ch = app.layout.content_height(&root);
                                        let max_scroll = (ch - wh).max(0.0);

                                        let margin = 20.0;
                                        if node_y < app.scroll_y {
                                            app.scroll_y = (node_y - margin).clamp(0.0, max_scroll);
                                        } else if node_y + node_h > app.scroll_y + wh {
                                            app.scroll_y = (node_y + node_h - wh + margin).clamp(0.0, max_scroll);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(w) = &app.window {
                    w.request_redraw();
                }
            }
            return;
        }

        if let Some(ref focused) = app.form.focused.clone() {
            app.caret_on = true;
            app.last_caret_toggle = std::time::Instant::now();

            if app.modifiers.control_key() {
                if let Key::Character(ref c) = event.logical_key {
                    match c.as_str() {
                        "c" | "C" => {
                            if let Some(selected) = get_selected_text(&app.form, focused) {
                                if let Ok(mut cb) = arboard::Clipboard::new() {
                                    let _ = cb.set_text(selected);
                                }
                            }
                            return;
                        }
                        "v" | "V" => {
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                if let Ok(text) = cb.get_text() {
                                    insert_text(&mut app.form, focused, &text);
                                    if let Some(w) = &app.window { w.request_redraw(); }
                                }
                            }
                            return;
                        }
                        "x" | "X" => {
                            if let Some(selected) = get_selected_text(&app.form, focused) {
                                if let Ok(mut cb) = arboard::Clipboard::new() {
                                    if cb.set_text(selected).is_ok() {
                                        delete_selected_text(&mut app.form, focused);
                                        if let Some(w) = &app.window { w.request_redraw(); }
                                    }
                                }
                            }
                            return;
                        }
                        "a" | "A" => {
                            app.form.select_all(focused);
                            if let Some(w) = &app.window { w.request_redraw(); }
                            return;
                        }
                        _ => {}
                    }
                }
            }

            match &event.logical_key {
                Key::Named(NamedKey::Backspace) => {
                    if !delete_selected_text(&mut app.form, focused) {
                        let len = app.form.get_value(focused).chars().count();
                        if len > 0 {
                            let pos = app.form.cursor(focused).min(len);
                            if pos > 0 {
                                let mut chars: Vec<char> = app.form.get_value(focused).chars().collect();
                                chars.remove(pos - 1);
                                let new_val: String = chars.into_iter().collect();
                                app.form.set_value(focused, new_val);
                                app.form.set_cursor(focused, pos - 1);
                            }
                        }
                    }
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Named(NamedKey::Enter) => {
                    if app.is_focused_textarea() {
                        delete_selected_text(&mut app.form, focused);
                        let len = app.form.get_value(focused).chars().count();
                        let pos = app.form.cursor(focused).min(len);
                        let mut chars: Vec<char> = app.form.get_value(focused).chars().collect();
                        chars.insert(pos, '\n');
                        let new_val: String = chars.into_iter().collect();
                        app.form.set_value(focused, new_val);
                        app.form.set_cursor(focused, pos + 1);
                        if let Some(w) = &app.window { w.request_redraw(); }
                    }
                }
                Key::Named(NamedKey::Escape) => {
                    app.focus_node(None);
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    let pos = app.form.cursor(focused);
                    if pos > 0 {
                        app.form.set_cursor(focused, pos - 1);
                        app.form.set_selection(focused, pos - 1, pos - 1);
                    }
                }
                Key::Named(NamedKey::ArrowRight) => {
                    let pos = app.form.cursor(focused);
                    let len = app.form.get_value(focused).chars().count();
                    if pos < len {
                        app.form.set_cursor(focused, pos + 1);
                        app.form.set_selection(focused, pos + 1, pos + 1);
                    }
                }
                Key::Named(NamedKey::Space) => {
                    delete_selected_text(&mut app.form, focused);
                    let len = app.form.get_value(focused).chars().count();
                    let pos = app.form.cursor(focused).min(len);
                    let mut chars: Vec<char> = app.form.get_value(focused).chars().collect();
                    chars.insert(pos, ' ');
                    let new_val: String = chars.into_iter().collect();
                    app.form.set_value(focused, new_val);
                    app.form.set_cursor(focused, pos + 1);
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Character(c) if c.len() == 1 && c.as_bytes()[0] >= 32 => {
                    delete_selected_text(&mut app.form, focused);
                    let len = app.form.get_value(focused).chars().count();
                    let pos = app.form.cursor(focused).min(len);
                    let mut chars: Vec<char> = app.form.get_value(focused).chars().collect();
                    chars.insert(pos, c.chars().next().unwrap());
                    let new_val: String = chars.into_iter().collect();
                    app.form.set_value(focused, new_val);
                    app.form.set_cursor(focused, pos + 1);
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                _ => {}
            }

            app.scroll_to_caret(focused);
            if let Some(w) = &app.window { w.request_redraw(); }
        } else {
            let win = match &app.window { Some(w) => w.clone(), None => return };
            let wh = win.inner_size().height.max(100) as f32;
            let ch = app.dom.as_ref().and_then(|d| d.document_element()).map(|r| app.layout.content_height(&r)).unwrap_or(0.0);
            let max_scroll = (ch - wh).max(0.0);

            match &event.logical_key {
                Key::Named(NamedKey::ArrowUp) => {
                    app.scroll_y = (app.scroll_y - 30.0).clamp(0.0, max_scroll);
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Named(NamedKey::ArrowDown) => {
                    app.scroll_y = (app.scroll_y + 30.0).clamp(0.0, max_scroll);
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Named(NamedKey::PageUp) => {
                    app.scroll_y = (app.scroll_y - 300.0).clamp(0.0, max_scroll);
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Named(NamedKey::PageDown) => {
                    app.scroll_y = (app.scroll_y + 300.0).clamp(0.0, max_scroll);
                    if let Some(w) = &app.window { w.request_redraw(); }
                }

                _ => {}
            }
        }
    }
}

pub(crate) fn handle_mouse_input(app: &mut App, state: ElementState, button: MouseButton) {
    if let Some(modal) = app.modal.clone() {
        if button == MouseButton::Left && state == ElementState::Pressed {
            let win = match &app.window { Some(w) => w.clone(), None => return };
            let size = win.inner_size();
            let vw = size.width.max(100) as f32;
            let vh = size.height.max(100) as f32;

            let mw = 420.0;
            let mh = 220.0;
            let mx = (vw - mw) / 2.0;
            let my = (vh - mh) / 2.0;

            let btn_w = 100.0;
            let btn_h = 36.0;
            let btn_y = my + mh - 24.0 - btn_h;
            let accept_x = mx + mw - 24.0 - btn_w;
            let cancel_x = accept_x - 12.0 - btn_w;

            if app.mouse_x >= accept_x && app.mouse_x < accept_x + btn_w && app.mouse_y >= btn_y && app.mouse_y < btn_y + btn_h {
                if modal.action == "confirm_submit" {
                    if let (Some(html), Some(css)) = (app.initial_html.clone(), app.initial_css.clone()) {
                        app.load(&html, &css);
                    }
                    app.modal = Some(ModalState {
                        title: "Formulario Enviado".to_string(),
                        message: "¡Los datos se han procesado correctamente y la operación fue un éxito!".to_string(),
                        modal_type: ModalType::Alert,
                        action: "submit_success".to_string(),
                    });
                    app.focus_node(None);
                } else {
                    app.modal = None;
                }
                if let Some(w) = &app.window {
                    w.request_redraw();
                }
            } else if modal.modal_type == ModalType::Confirm && app.mouse_x >= cancel_x && app.mouse_x < cancel_x + btn_w && app.mouse_y >= btn_y && app.mouse_y < btn_y + btn_h {
                app.modal = None;
                if let Some(w) = &app.window {
                    w.request_redraw();
                }
            }
        }
        return;
    }
    if button == MouseButton::Left {
        let win = match &app.window { Some(w) => w.clone(), None => return };
        let size = win.inner_size();
        let ww = size.width.max(100) as f32;
        let wh = size.height.max(100) as f32;
        let ch = app.dom.as_ref().and_then(|d| d.document_element()).map(|r| app.layout.content_height(&r)).unwrap_or(0.0);
        let sb_w = 10.0;
        let sb_x = ww - sb_w - 2.0;
        let track_h = wh - 4.0;
        let thumb_h = (wh / ch * track_h).max(20.0);
        let max_scroll = (ch - wh).max(0.0);

        match state {
            ElementState::Pressed => {
                let now = std::time::Instant::now();
                let dt = now - app.last_click_time;
                if dt.as_millis() < 400 && (app.mouse_x - app.last_click_pos.0).abs() < 8.0 && (app.mouse_y - app.last_click_pos.1).abs() < 8.0 {
                    app.click_count += 1;
                } else { app.click_count = 1; }
                app.last_click_time = now;
                app.last_click_pos = (app.mouse_x, app.mouse_y);

                let mut clicked_dropdown = false;
                if let Some(ref focused_key) = app.form.focused.clone() {
                    if let Some(ref dom) = app.dom {
                        if let Some(root) = dom.document_element() {
                            fn find_node(node: &std::rc::Rc<dom::Node>, target_key: &str) -> Option<std::rc::Rc<dom::Node>> {
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
                            if let Some(focused_node) = find_node(&root, focused_key) {
                                if focused_node.tag_name() == Some("select") {
                                    if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                                        if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                            let sw = lr.size.width;
                                            let sh = lr.size.height;
                                            
                                            let mut options = Vec::new();
                                            for child in &focused_node.children {
                                                if child.tag_name() == Some("option") {
                                                    options.push(child.clone());
                                                }
                                            }
                                            
                                            let opt_h = 30.0;
                                            let max_dropdown_h = if let Some(style) = app.styles.get(&dom::node_ptr(&focused_node)) {
                                                match style.max_height {
                                                    toldo_ui_engine::style::Length::Px(v) => v,
                                                    _ => 7.0 * opt_h,
                                                }
                                            } else {
                                                7.0 * opt_h
                                            };
                                            let total_h = options.len() as f32 * opt_h;
                                            let dropdown_h = total_h.min(max_dropdown_h);
                                            let dropdown_scroll = app.form.get_dropdown_scroll_y(focused_key);
                                            
                                            if app.mouse_x >= sx && app.mouse_x < sx + sw && app.mouse_y >= sy + sh && app.mouse_y < sy + sh + dropdown_h {
                                                let clicked_idx = ((app.mouse_y + dropdown_scroll - (sy + sh)) / opt_h) as usize;
                                                if clicked_idx < options.len() {
                                                    let selected_option = &options[clicked_idx];
                                                    let option_text = selected_option.children_text().trim().to_string();
                                                    app.form.set_value(focused_key, option_text);
                                                    
                                                    for opt in &options {
                                                        let opt_key = format!("{:p}", dom::node_ptr(opt));
                                                        app.form.checked.insert(opt_key, false);
                                                    }
                                                    let selected_opt_key = format!("{:p}", dom::node_ptr(selected_option));
                                                    app.form.checked.insert(selected_opt_key, true);
                                                }
                                                app.focus_node(None);
                                                clicked_dropdown = true;
                                                if let Some(w) = &app.window { w.request_redraw(); }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if clicked_dropdown {
                    // Click consumed by dropdown
                } else if let Some((node, mut form_type)) = app.hit_test(app.mouse_x, app.mouse_y + app.scroll_y) {
                    let mut click_node = node.clone();
                    
                    // Walk up to find if there is an ancestor label
                    let mut label_ancestor = None;
                    {
                        let mut curr: *const dom::Node = dom::node_ptr(&node);
                        unsafe {
                            while !curr.is_null() {
                                if (*curr).tag_name() == Some("label") {
                                    label_ancestor = Some(&*curr);
                                    break;
                                }
                                if let Some(parent) = (*curr).get_parent() {
                                    curr = parent as *const dom::Node;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    
                    if let Some(label_node) = label_ancestor {
                        fn find_input_child(n: &dom::Node) -> Option<std::rc::Rc<dom::Node>> {
                            for child in &n.children {
                                if child.tag_name() == Some("input") {
                                    let itype = child.get_attribute("type").unwrap_or("");
                                    if itype == "checkbox" || itype == "radio" {
                                        return Some(child.clone());
                                    }
                                }
                                if let Some(found) = find_input_child(child) {
                                    return Some(found);
                                }
                            }
                            None
                        }
                        if let Some(input_node) = find_input_child(label_node) {
                            let itype = input_node.get_attribute("type").unwrap_or("");
                            form_type = if itype == "checkbox" { "checkbox" } else { "radio" };
                            click_node = input_node;
                        }
                    }
                    
                    let key = format!("{:p}", dom::node_ptr(&click_node));
                    match form_type {
                        "button" => {
                            let mut listeners = std::mem::take(&mut app.click_listeners);
                            for listener in &mut listeners {
                                if crate::core::app::matches_selector(&click_node, &listener.selector) {
                                    (listener.callback)(app, &click_node);
                                }
                            }
                            app.click_listeners.extend(listeners);

                            app.focus_node(None);
                            app.click_count = 0;
                            if let Some(w) = &app.window { w.request_redraw(); }
                        }
                        "checkbox" => { app.form.toggle(&key); app.focus_node(None); app.click_count = 0; }
                        "radio" => {
                            if !app.form.is_checked(&key) {
                                app.form.checked.insert(key.clone(), true);
                                if let Some(ref dom) = app.dom {
                                    if let Some(root) = dom.document_element() {
                                        let name_attr = click_node.get_attribute("name");
                                        fn deselect_other_radios(
                                            node: &std::rc::Rc<toldo_ui_engine::dom::Node>,
                                            target_key: &str,
                                            group_name: &str,
                                            form: &mut toldo_ui_engine::form::FormState,
                                        ) {
                                            let key = format!("{:p}", toldo_ui_engine::dom::node_ptr(node));
                                            if key != target_key && node.tag_name() == Some("input") {
                                                if node.get_attribute("type") == Some("radio") {
                                                    if node.get_attribute("name") == Some(group_name) {
                                                        form.checked.insert(key.clone(), false);
                                                    }
                                                }
                                            }
                                            for child in &node.children {
                                                deselect_other_radios(child, target_key, group_name, form);
                                            }
                                        }
                                        if let Some(name) = name_attr {
                                            if !name.is_empty() {
                                                deselect_other_radios(&root, &key, name, &mut app.form);
                                            }
                                        }
                                    }
                                }
                            }
                            app.focus_node(None);
                            app.click_count = 0;
                        }
                        "select" => {
                            if app.form.focused.as_ref() == Some(&key) {
                                app.focus_node(None);
                            } else {
                                app.focus_node(Some(key.clone()));
                            }
                            app.click_count = 0;
                        }
                        "text" | "textarea" => {
                            app.focus_node(Some(key.clone()));
                            let val = app.form.get_value(&key);
                            let mut start_idx = val.chars().count();
                            let node_ptr = dom::node_ptr(&node);
                            if let Some(root) = app.dom.as_ref().and_then(|d| d.document_element()) {
                                if let Some((node_x, node_y)) = get_node_abs_pos(&root, node_ptr, &app.layout, 0.0, 0.0) {
                                    if let Some(style) = app.styles.get(&node_ptr) {
                                        let padding_left = match style.padding_left { toldo_ui_engine::style::Length::Px(v) => v, _ => 0.0 };
                                        let border_left = style.border.left.width;
                                        let cx = node_x + padding_left + border_left;
                                        let target_x = app.mouse_x - cx;
                                        
                                        if form_type == "textarea" {
                                            let padding_top = match style.padding_top { toldo_ui_engine::style::Length::Px(v) => v, _ => 0.0 };
                                            let border_top = style.border.top.width;
                                            let cy = node_y + padding_top + border_top;
                                            let target_y = app.mouse_y + app.scroll_y - cy;
                                            
                                            let padding_right = match style.padding_right { toldo_ui_engine::style::Length::Px(v) => v, _ => 0.0 };
                                            let border_right = style.border.right.width;
                                            let node_w = app.layout.get(node_ptr).map(|l| l.size.width).unwrap_or(0.0);
                                            let max_w = node_w - padding_left - padding_right - border_left - border_right - 2.0;
                                            
                                            start_idx = toldo_ui_engine::render::painter::textarea_index_at_point(
                                                style,
                                                val,
                                                target_x,
                                                target_y,
                                                max_w,
                                            );
                                        } else {
                                            start_idx = toldo_ui_engine::render::painter::index_at_x(style, val, target_x);
                                        }
                                    }
                                }
                            }
                            if app.click_count >= 2 {
                                app.form.select_all(&key);
                            } else {
                                app.form.set_cursor(&key, start_idx);
                                app.form.set_selection(&key, start_idx, start_idx);
                            }
                            app.caret_on = true;
                            app.last_caret_toggle = std::time::Instant::now();
                            app.dragging_select = true;
                            app.dragging_node = Some(node_ptr);
                        }
                        _ => {
                            app.focus_node(None);
                            app.click_count = 0;
                        }
                    }
                    if let Some(w) = &app.window { w.request_redraw(); }
                } else {
                    app.focus_node(None);
                    app.click_count = 0;
                }

                if app.mouse_y >= 2.0 && app.mouse_y < 2.0 + track_h && app.mouse_x >= sb_x && app.mouse_x < sb_x + sb_w {
                    let thumb_y = if max_scroll > 0.0 { 2.0 + (app.scroll_y / max_scroll) * (track_h - thumb_h) } else { 2.0 };
                    if app.mouse_y >= thumb_y && app.mouse_y < thumb_y + thumb_h {
                        app.dragging_scrollbar = true;
                    } else {
                        let ratio = ((app.mouse_y - 2.0) - thumb_h * 0.5) / (track_h - thumb_h);
                        let val = (ratio * max_scroll).clamp(0.0, max_scroll);
                        app.scroll_y = val;
                        if let Some(w) = &app.window { w.request_redraw(); }
                    }
                }
            }
            ElementState::Released => {
                app.dragging_scrollbar = false;
                app.dragging_select = false;
                app.dragging_node = None;
            }
        }
        app.update_cursor_icon();
        app.update_hover();
    }
}

pub(crate) fn handle_cursor_moved(app: &mut App, position: PhysicalPosition<f64>) {
    app.mouse_x = position.x as f32;
    app.mouse_y = position.y as f32;

    if app.modal.is_some() {
        if let Some(w) = &app.window {
            w.request_redraw();
        }
        return;
    }

    if app.dragging_scrollbar {
        let win = match &app.window { Some(w) => w.clone(), None => return };
        let size = win.inner_size();
        let wh = size.height.max(100) as f32;
        let ch = app.dom.as_ref().and_then(|d| d.document_element()).map(|r| app.layout.content_height(&r)).unwrap_or(0.0);
        let max_scroll = (ch - wh).max(0.0);
        let track_h = wh - 4.0;
        let thumb_h = (wh / ch * track_h).max(20.0);
        let ratio = (app.mouse_y - 2.0 - thumb_h * 0.5) / (track_h - thumb_h);
        let val = (ratio * max_scroll).clamp(0.0, max_scroll);
        app.scroll_y = val;
        if let Some(w) = &app.window { w.request_redraw(); }
    }

    if app.dragging_select {
        if let Some(node_ptr) = app.dragging_node {
            let key = format!("{:p}", node_ptr);
            if let Some(root) = app.dom.as_ref().and_then(|d| d.document_element()) {
                if let Some((node_x, node_y)) = get_node_abs_pos(&root, node_ptr, &app.layout, 0.0, 0.0) {
                    if let Some(style) = app.styles.get(&node_ptr) {
                        let padding_left = match style.padding_left { toldo_ui_engine::style::Length::Px(v) => v, _ => 0.0 };
                        let border_left = style.border.left.width;
                        let cx = node_x + padding_left + border_left;
                        let val = app.form.get_value(&key);
                        let target_x = app.mouse_x - cx;
                        
                        let is_textarea = unsafe { (*node_ptr).tag_name() == Some("textarea") };
                        let current_idx = if is_textarea {
                            let padding_top = match style.padding_top { toldo_ui_engine::style::Length::Px(v) => v, _ => 0.0 };
                            let border_top = style.border.top.width;
                            let cy = node_y + padding_top + border_top;
                            let target_y = app.mouse_y + app.scroll_y - cy;
                            
                            let padding_right = match style.padding_right { toldo_ui_engine::style::Length::Px(v) => v, _ => 0.0 };
                            let border_right = style.border.right.width;
                            let node_w = app.layout.get(node_ptr).map(|l| l.size.width).unwrap_or(0.0);
                            let max_w = node_w - padding_left - padding_right - border_left - border_right - 2.0;
                            
                            toldo_ui_engine::render::painter::textarea_index_at_point(
                                style,
                                val,
                                target_x,
                                target_y,
                                max_w,
                            )
                        } else {
                            toldo_ui_engine::render::painter::index_at_x(style, val, target_x)
                        };

                        if let Some((start_idx, _)) = app.form.get_selection(&key) {
                            app.form.set_selection(&key, start_idx, current_idx);
                            app.form.set_cursor(&key, current_idx);
                            if let Some(w) = &app.window { w.request_redraw(); }
                        }
                    }
                }
            }
        }
    }

    app.update_cursor_icon();
    let mut needs_redraw = app.update_hover();
    
    let mut dropdown_hover = None;
    if let Some(ref focused_key) = app.form.focused {
        if let Some(ref dom) = app.dom {
            if let Some(root) = dom.document_element() {
                fn find_select(node: &std::rc::Rc<toldo_ui_engine::dom::Node>, target_key: &str) -> Option<std::rc::Rc<toldo_ui_engine::dom::Node>> {
                    let key = format!("{:p}", toldo_ui_engine::dom::node_ptr(node));
                    if key == target_key {
                        if node.tag_name() == Some("select") {
                            return Some(node.clone());
                        }
                    }
                    for child in &node.children {
                        if let Some(n) = find_select(child, target_key) {
                            return Some(n);
                        }
                    }
                    None
                }
                if let Some(focused_node) = find_select(&root, focused_key) {
                    if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                        if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                            let sw = lr.size.width;
                            let sh = lr.size.height;
                            
                            let mut options = Vec::new();
                            for child in &focused_node.children {
                                if child.tag_name() == Some("option") {
                                    options.push(child.clone());
                                }
                            }
                            
                            let opt_h = 30.0;
                            let max_dropdown_h = if let Some(style) = app.styles.get(&dom::node_ptr(&focused_node)) {
                                match style.max_height {
                                    toldo_ui_engine::style::Length::Px(v) => v,
                                    _ => 7.0 * opt_h,
                                }
                            } else {
                                7.0 * opt_h
                            };
                            let total_h = options.len() as f32 * opt_h;
                            let dropdown_h = total_h.min(max_dropdown_h);
                            let dropdown_scroll = app.form.get_dropdown_scroll_y(focused_key);
                            
                            if app.mouse_x >= sx && app.mouse_x < sx + sw && app.mouse_y >= sy + sh && app.mouse_y < sy + sh + dropdown_h {
                                let clicked_idx = ((app.mouse_y + dropdown_scroll - (sy + sh)) / opt_h) as usize;
                                if clicked_idx < options.len() {
                                    dropdown_hover = Some(clicked_idx);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    if dropdown_hover != app.last_dropdown_hover {
        app.last_dropdown_hover = dropdown_hover;
        needs_redraw = true;
    }
    
    if needs_redraw {
        if let Some(w) = &app.window { w.request_redraw(); }
    }
}

pub(crate) fn handle_mouse_wheel(app: &mut App, delta: MouseScrollDelta) {
    if app.modal.is_some() {
        return;
    }
    let mut scrolled_dropdown = false;
    if let Some(ref focused_key) = app.form.focused.clone() {
        if let Some(ref dom) = app.dom {
            if let Some(root) = dom.document_element() {
                fn find_node(node: &std::rc::Rc<dom::Node>, target_key: &str) -> Option<std::rc::Rc<dom::Node>> {
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
                if let Some(focused_node) = find_node(&root, focused_key) {
                    if focused_node.tag_name() == Some("select") {
                        if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                            if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                let sw = lr.size.width;
                                let sh = lr.size.height;
                                
                                let mut options = Vec::new();
                                for child in &focused_node.children {
                                    if child.tag_name() == Some("option") {
                                        options.push(child.clone());
                                    }
                                }
                                
                                let opt_h = 30.0;
                                let max_dropdown_h = if let Some(style) = app.styles.get(&dom::node_ptr(&focused_node)) {
                                    match style.max_height {
                                        toldo_ui_engine::style::Length::Px(v) => v,
                                        _ => 7.0 * opt_h,
                                    }
                                } else {
                                    7.0 * opt_h
                                };
                                let total_h = options.len() as f32 * opt_h;
                                let dropdown_h = total_h.min(max_dropdown_h);
                                
                                if app.mouse_x >= sx && app.mouse_x < sx + sw && app.mouse_y >= sy + sh && app.mouse_y < sy + sh + dropdown_h {
                                    let scroll_y = app.form.get_dropdown_scroll_y(focused_key);
                                    let dy = match delta {
                                        MouseScrollDelta::LineDelta(_, y) => -y * 20.0,
                                        MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                                    };
                                    let max_scroll = (total_h - dropdown_h).max(0.0);
                                    let new_scroll = (scroll_y + dy).clamp(0.0, max_scroll);
                                    app.form.set_dropdown_scroll_y(focused_key, new_scroll);
                                    scrolled_dropdown = true;
                                    if let Some(w) = &app.window { w.request_redraw(); }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !scrolled_dropdown {
        let win = match &app.window { Some(w) => w.clone(), None => return };
        let wh = win.inner_size().height.max(100) as f32;
        let ch = app.dom.as_ref().and_then(|d| d.document_element()).map(|r| app.layout.content_height(&r)).unwrap_or(0.0);
        let max_scroll = (ch - wh).max(0.0);

        let dy = match delta {
            MouseScrollDelta::LineDelta(_, y) => -y * 70.0,
            MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
        };

        app.scroll_y = (app.scroll_y + dy).clamp(0.0, max_scroll);
        app.dragging_scrollbar = false;
        app.update_cursor_icon();
        app.update_hover();
        if let Some(w) = &app.window { w.request_redraw(); }
    }
}
