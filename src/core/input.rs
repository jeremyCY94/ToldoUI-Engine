use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::{Key, NamedKey};
use winit::dpi::PhysicalPosition;

use crate::dom;
use crate::form::actions::{delete_selected_text, get_selected_text, insert_text};
use crate::render::overlay::{ModalState, ModalType};
use crate::form::{DateSection, TimeSection};


use crate::core::app::{App, get_node_abs_pos};

pub fn get_time_section_and_bounds(format: &str, pos: usize) -> (TimeSection, usize, usize) {
    let format_lower = format.to_lowercase();
    if format_lower == "hh:mm:ss" {
        if pos < 2 {
            (TimeSection::Hour, 0, 2)
        } else if pos >= 3 && pos < 5 {
            (TimeSection::Minute, 3, 5)
        } else if pos >= 6 {
            (TimeSection::Second, 6, 8)
        } else {
            if pos == 2 {
                (TimeSection::Hour, 0, 2)
            } else {
                (TimeSection::Minute, 3, 5)
            }
        }
    } else {
        // HH:mm
        if pos < 2 {
            (TimeSection::Hour, 0, 2)
        } else if pos >= 3 && pos < 5 {
            (TimeSection::Minute, 3, 5)
        } else {
            if pos == 2 {
                (TimeSection::Hour, 0, 2)
            } else {
                (TimeSection::Minute, 3, 5)
            }
        }
    }
}

fn update_time_value_section(val: &str, format: &str, section: TimeSection, new_sec_val: &str) -> String {
    let mut chars: Vec<char> = val.chars().collect();
    let format_lower = format.to_lowercase();
    let new_chars: Vec<char> = new_sec_val.chars().collect();
    
    if format_lower == "hh:mm:ss" {
        match section {
            TimeSection::Hour => {
                if chars.len() >= 2 && new_chars.len() == 2 {
                    chars[0..2].copy_from_slice(&new_chars[0..2]);
                }
            }
            TimeSection::Minute => {
                if chars.len() >= 5 && new_chars.len() == 2 {
                    chars[3..5].copy_from_slice(&new_chars[0..2]);
                }
            }
            TimeSection::Second => {
                if chars.len() >= 8 && new_chars.len() == 2 {
                    chars[6..8].copy_from_slice(&new_chars[0..2]);
                }
            }
        }
    } else {
        // HH:mm
        match section {
            TimeSection::Hour => {
                if chars.len() >= 2 && new_chars.len() == 2 {
                    chars[0..2].copy_from_slice(&new_chars[0..2]);
                }
            }
            TimeSection::Minute => {
                if chars.len() >= 5 && new_chars.len() == 2 {
                    chars[3..5].copy_from_slice(&new_chars[0..2]);
                }
            }
            TimeSection::Second => {}
        }
    }
    chars.into_iter().collect()
}

pub fn get_date_section_and_bounds(format: &str, pos: usize) -> (DateSection, usize, usize) {
    let format_lower = format.to_lowercase();
    if format_lower == "yyyy-mm-dd" {
        if pos < 4 {
            (DateSection::Year, 0, 4)
        } else if pos >= 5 && pos < 7 {
            (DateSection::Month, 5, 7)
        } else if pos >= 8 {
            (DateSection::Day, 8, 10)
        } else {
            if pos == 4 {
                (DateSection::Year, 0, 4)
            } else {
                (DateSection::Month, 5, 7)
            }
        }
    } else {
        // dd/MM/yyyy
        if pos < 2 {
            (DateSection::Day, 0, 2)
        } else if pos >= 3 && pos < 5 {
            (DateSection::Month, 3, 5)
        } else if pos >= 6 {
            (DateSection::Year, 6, 10)
        } else {
            if pos == 2 {
                (DateSection::Day, 0, 2)
            } else {
                (DateSection::Month, 3, 5)
            }
        }
    }
}


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
                        fn find_node_ptr(node: &std::rc::Rc<crate::dom::Node>, target_key: &str) -> Option<*const crate::dom::Node> {
                            let key = format!("{:p}", crate::dom::node_ptr(node));
                            if key == target_key {
                                return Some(crate::dom::node_ptr(node));
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

            let mut is_date_input = false;
            let mut date_format = String::new();
            let mut is_time_input = false;
            let mut time_format = String::new();
            if let Some(ref dom) = app.dom {
                if let Some(root) = dom.document_element() {
                    if let Some(node) = dom::Node::find_node_by_key(&root, focused) {
                        if node.tag_name() == Some("input") {
                            let itype = node.get_attribute("type").unwrap_or("text");
                            if itype == "date" {
                                is_date_input = true;
                                date_format = node.get_attribute("format").unwrap_or("dd/MM/yyyy").to_string();
                            } else if itype == "time" {
                                is_time_input = true;
                                time_format = node.get_attribute("format").unwrap_or("HH:mm").to_string();
                            }
                        }
                    }
                }
            }

            if is_date_input {
                let mut val = app.form.get_value(focused).to_string();
                if val.is_empty() {
                    val = date_format.clone();
                    app.form.set_value(focused, val.clone());
                }
                let pos = app.form.cursor(focused);
                
                match &event.logical_key {
                    Key::Named(NamedKey::Backspace) => {
                        let mut idx = pos;
                        while idx > 0 {
                            idx -= 1;
                            let c = val.chars().nth(idx).unwrap_or(' ');
                            if c != '/' && c != '-' {
                                let template_char = date_format.chars().nth(idx).unwrap_or(c);
                                let mut chars: Vec<char> = val.chars().collect();
                                if idx < chars.len() {
                                    chars[idx] = template_char;
                                    let new_val: String = chars.into_iter().collect();
                                    app.form.set_value(focused, new_val);
                                    app.form.set_cursor(focused, idx);
                                    let active_sec = get_date_section_and_bounds(&date_format, idx).0;
                                    app.form.set_date_active_section(focused, active_sec);
                                }
                                break;
                            }
                        }
                        if let Some(w) = &app.window { w.request_redraw(); }
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        if pos > 0 {
                            let new_pos = pos - 1;
                            app.form.set_cursor(focused, new_pos);
                            app.form.set_selection(focused, new_pos, new_pos);
                            let active_sec = get_date_section_and_bounds(&date_format, new_pos).0;
                            app.form.set_date_active_section(focused, active_sec);
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if pos < val.chars().count() {
                            let new_pos = pos + 1;
                            app.form.set_cursor(focused, new_pos);
                            app.form.set_selection(focused, new_pos, new_pos);
                            let active_sec = get_date_section_and_bounds(&date_format, new_pos.min(val.chars().count() - 1)).0;
                            app.form.set_date_active_section(focused, active_sec);
                        }
                    }
                    Key::Character(c) if c.len() == 1 && c.chars().next().unwrap().is_ascii_digit() => {
                        let digit = c.chars().next().unwrap();
                        let mut idx = pos;
                        while idx < val.chars().count() {
                            let ch = val.chars().nth(idx).unwrap_or(' ');
                            if ch != '/' && ch != '-' {
                                let mut chars: Vec<char> = val.chars().collect();
                                chars[idx] = digit;
                                let new_val: String = chars.into_iter().collect();
                                app.form.set_value(focused, new_val);
                                
                                let mut next_pos = idx + 1;
                                if next_pos < date_format.chars().count() {
                                    let next_c = date_format.chars().nth(next_pos).unwrap();
                                    if next_c == '/' || next_c == '-' {
                                        next_pos += 1;
                                    }
                                }
                                app.form.set_cursor(focused, next_pos.min(date_format.chars().count()));
                                
                                let active_sec = get_date_section_and_bounds(&date_format, next_pos.min(date_format.chars().count() - 1)).0;
                                app.form.set_date_active_section(focused, active_sec);
                                break;
                            }
                            idx += 1;
                        }
                        if let Some(w) = &app.window { w.request_redraw(); }
                    }
                    Key::Named(NamedKey::Escape) => {
                        app.focus_node(None);
                        if let Some(w) = &app.window { w.request_redraw(); }
                    }
                    _ => {}
                }
                
                app.scroll_to_caret(focused);
                if let Some(w) = &app.window { w.request_redraw(); }
                return;
            }

            if is_time_input {
                let mut val = app.form.get_value(focused).to_string();
                if val.is_empty() {
                    val = time_format.clone();
                    app.form.set_value(focused, val.clone());
                }
                let pos = app.form.cursor(focused);
                
                match &event.logical_key {
                    Key::Named(NamedKey::Backspace) => {
                        let mut idx = pos;
                        while idx > 0 {
                            idx -= 1;
                            let c = val.chars().nth(idx).unwrap_or(' ');
                            if c != ':' {
                                let template_char = time_format.chars().nth(idx).unwrap_or(c);
                                let mut chars: Vec<char> = val.chars().collect();
                                if idx < chars.len() {
                                    chars[idx] = template_char;
                                    let new_val: String = chars.into_iter().collect();
                                    app.form.set_value(focused, new_val);
                                    app.form.set_cursor(focused, idx);
                                    let active_sec = get_time_section_and_bounds(&time_format, idx).0;
                                    app.form.set_time_active_section(focused, active_sec);
                                }
                                break;
                            }
                        }
                        if let Some(w) = &app.window { w.request_redraw(); }
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        if pos > 0 {
                            let new_pos = pos - 1;
                            app.form.set_cursor(focused, new_pos);
                            app.form.set_selection(focused, new_pos, new_pos);
                            let active_sec = get_time_section_and_bounds(&time_format, new_pos).0;
                            app.form.set_time_active_section(focused, active_sec);
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if pos < val.chars().count() {
                            let new_pos = pos + 1;
                            app.form.set_cursor(focused, new_pos);
                            app.form.set_selection(focused, new_pos, new_pos);
                            let active_sec = get_time_section_and_bounds(&time_format, new_pos.min(val.chars().count() - 1)).0;
                            app.form.set_time_active_section(focused, active_sec);
                        }
                    }
                    Key::Character(c) if c.len() == 1 && c.chars().next().unwrap().is_ascii_digit() => {
                        let digit = c.chars().next().unwrap();
                        let mut idx = pos;
                        while idx < val.chars().count() {
                            let ch = val.chars().nth(idx).unwrap_or(' ');
                            if ch != ':' {
                                let mut chars: Vec<char> = val.chars().collect();
                                chars[idx] = digit;
                                let new_val: String = chars.into_iter().collect();
                                app.form.set_value(focused, new_val);
                                
                                let mut next_pos = idx + 1;
                                if next_pos < time_format.chars().count() {
                                    let next_c = time_format.chars().nth(next_pos).unwrap();
                                    if next_c == ':' {
                                        next_pos += 1;
                                    }
                                }
                                app.form.set_cursor(focused, next_pos.min(time_format.chars().count()));
                                
                                let active_sec = get_time_section_and_bounds(&time_format, next_pos.min(time_format.chars().count() - 1)).0;
                                app.form.set_time_active_section(focused, active_sec);
                                break;
                            }
                            idx += 1;
                        }
                        if let Some(w) = &app.window { w.request_redraw(); }
                    }
                    Key::Named(NamedKey::Escape) => {
                        app.focus_node(None);
                        if let Some(w) = &app.window { w.request_redraw(); }
                    }
                    _ => {}
                }
                
                app.scroll_to_caret(focused);
                if let Some(w) = &app.window { w.request_redraw(); }
                return;
            }

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
                app.last_click_pos = (app.mouse_x, app.mouse_y);                let mut clicked_dropdown = false;
                if let Some(ref focused_key) = app.form.focused.clone() {
                    if let Some(ref dom) = app.dom {
                        if let Some(root) = dom.document_element() {
                            if let Some(focused_node) = dom::Node::find_node_by_key(&root, focused_key) {
                                if focused_node.tag_name() == Some("select") {
                                    if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                                        if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                            let sw = lr.size.width;
                                            let sh = lr.size.height;
                                            clicked_dropdown = handle_select_click(app, focused_key, &focused_node, sx, sy, sw, sh);
                                        }
                                    }
                                } else if focused_node.tag_name() == Some("input") && focused_node.get_attribute("type") == Some("date") {
                                    if app.form.is_date_picker_open(focused_key) {
                                        if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                                            clicked_dropdown = handle_date_picker_click(app, focused_key, &focused_node, sx, sy);
                                        }
                                    }
                                } else if focused_node.tag_name() == Some("input") && focused_node.get_attribute("type") == Some("time") {
                                    if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                                        if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                            let sw = lr.size.width;
                                            let sh = lr.size.height;
                                            clicked_dropdown = handle_time_picker_click(app, focused_key, &focused_node, sx, sy, sw, sh);
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
                        let mut curr = Some(&*node);
                        while let Some(n) = curr {
                            if n.tag_name() == Some("label") {
                                label_ancestor = Some(n);
                                break;
                            }
                            curr = n.get_parent();
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
                                            node: &std::rc::Rc<crate::dom::Node>,
                                            target_key: &str,
                                            group_name: &str,
                                            form: &mut crate::form::FormState,
                                        ) {
                                            let key = format!("{:p}", crate::dom::node_ptr(node));
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
                                        let padding_left = match style.padding_left { crate::style::Length::Px(v) => v, _ => 0.0 };
                                        let border_left = style.border.left.width;
                                        let cx = node_x + padding_left + border_left;
                                        let target_x = app.mouse_x - cx;
                                        
                                        if form_type == "textarea" {
                                            let padding_top = match style.padding_top { crate::style::Length::Px(v) => v, _ => 0.0 };
                                            let border_top = style.border.top.width;
                                            let cy = node_y + padding_top + border_top;
                                            let target_y = app.mouse_y + app.scroll_y - cy;
                                            
                                            let padding_right = match style.padding_right { crate::style::Length::Px(v) => v, _ => 0.0 };
                                            let border_right = style.border.right.width;
                                            let node_w = app.layout.get(node_ptr).map(|l| l.size.width).unwrap_or(0.0);
                                            let max_w = node_w - padding_left - padding_right - border_left - border_right - 2.0;
                                            
                                            start_idx = crate::render::painter::textarea_index_at_point(
                                                style,
                                                val,
                                                target_x,
                                                target_y,
                                                max_w,
                                            );
                                        } else {
                                            start_idx = crate::render::painter::index_at_x(style, val, target_x);
                                        }
                                    }
                                }
                            }
                            let is_date = node.tag_name() == Some("input") && node.get_attribute("type") == Some("date");
                            let is_time = node.tag_name() == Some("input") && node.get_attribute("type") == Some("time");
                            if is_date {
                                let mut click_on_icon = false;
                                if let Some(root) = app.dom.as_ref().and_then(|d| d.document_element()) {
                                    if let Some((node_x, _node_y)) = get_node_abs_pos(&root, node_ptr, &app.layout, 0.0, 0.0) {
                                        if let Some(style) = app.styles.get(&node_ptr) {
                                            let padding_left = match style.padding_left { crate::style::Length::Px(v) => v, _ => 0.0 };
                                            let border_left = style.border.left.width;
                                            let icon_start = node_x + padding_left + border_left;
                                            let icon_end = icon_start + 24.0;
                                            if app.mouse_x >= icon_start && app.mouse_x < icon_end {
                                                click_on_icon = true;
                                            }
                                        }
                                    }
                                }

                                if click_on_icon {
                                    let current_open = app.form.is_date_picker_open(&key);
                                    app.form.set_date_picker_open(&key, !current_open);
                                    if !current_open {
                                        let val = app.form.get_value(&key);
                                        let format = node.get_attribute("format").unwrap_or("dd/MM/yyyy");
                                        let (m, y) = if let Some((_, parse_m, parse_y)) = crate::form::parse_date_value(val, format) {
                                            (parse_m, parse_y)
                                        } else {
                                            crate::form::get_current_month_year()
                                        };
                                        app.form.set_date_picker_month_year(&key, m, y);
                                    }
                                } else {
                                    app.form.set_date_picker_open(&key, false);
                                    let format = node.get_attribute("format").unwrap_or("dd/MM/yyyy");
                                    let mut val = app.form.get_value(&key).to_string();
                                    if val.is_empty() {
                                        val = format.to_string();
                                        app.form.set_value(&key, val.clone());
                                    }
                                    let clamped_idx = start_idx.min(val.chars().count());
                                    app.form.set_cursor(&key, clamped_idx);
                                    app.form.set_selection(&key, clamped_idx, clamped_idx);
                                    
                                    let active_sec = get_date_section_and_bounds(format, clamped_idx).0;
                                    app.form.set_date_active_section(&key, active_sec);
                                    app.form.set_dropdown_scroll_y(&key, 0.0);
                                    
                                    let current_yr = crate::form::get_current_year();
                                    app.form.set_date_years_range(&key, (current_yr - 10, current_yr + 10));
                                    
                                    app.caret_on = true;
                                    app.last_caret_toggle = std::time::Instant::now();
                                }
                            } else if is_time {
                                let format = node.get_attribute("format").unwrap_or("HH:mm");
                                let mut val = app.form.get_value(&key).to_string();
                                if val.is_empty() {
                                    val = format.to_string();
                                    app.form.set_value(&key, val.clone());
                                }
                                let clamped_idx = start_idx.min(val.chars().count());
                                app.form.set_cursor(&key, clamped_idx);
                                app.form.set_selection(&key, clamped_idx, clamped_idx);
                                
                                let active_sec = get_time_section_and_bounds(format, clamped_idx).0;
                                app.form.set_time_active_section(&key, active_sec);
                                app.form.set_dropdown_scroll_y(&key, 0.0);
                                
                                app.caret_on = true;
                                app.last_caret_toggle = std::time::Instant::now();
                            } else {
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
                        let padding_left = match style.padding_left { crate::style::Length::Px(v) => v, _ => 0.0 };
                        let border_left = style.border.left.width;
                        let cx = node_x + padding_left + border_left;
                        let val = app.form.get_value(&key);
                        let target_x = app.mouse_x - cx;
                        
                        let is_textarea = unsafe { (*node_ptr).tag_name() == Some("textarea") };
                        let current_idx = if is_textarea {
                            let padding_top = match style.padding_top { crate::style::Length::Px(v) => v, _ => 0.0 };
                            let border_top = style.border.top.width;
                            let cy = node_y + padding_top + border_top;
                            let target_y = app.mouse_y + app.scroll_y - cy;
                            
                            let padding_right = match style.padding_right { crate::style::Length::Px(v) => v, _ => 0.0 };
                            let border_right = style.border.right.width;
                            let node_w = app.layout.get(node_ptr).map(|l| l.size.width).unwrap_or(0.0);
                            let max_w = node_w - padding_left - padding_right - border_left - border_right - 2.0;
                            
                            crate::render::painter::textarea_index_at_point(
                                style,
                                val,
                                target_x,
                                target_y,
                                max_w,
                            )
                        } else {
                            crate::render::painter::index_at_x(style, val, target_x)
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
    if let Some(focused_key) = app.form.focused.clone() {
        if let Some(ref dom) = app.dom {
            if let Some(root) = dom.document_element() {
                if let Some(focused_node) = dom::Node::find_node_by_key(&root, &focused_key) {
                    if focused_node.tag_name() == Some("select") {
                        if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                            if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                let sw = lr.size.width;
                                let sh = lr.size.height;
                                dropdown_hover = handle_select_hover(app, &focused_key, &focused_node, sx, sy, sw, sh);
                            }
                        }
                    } else if focused_node.tag_name() == Some("input") && focused_node.get_attribute("type") == Some("date") {
                        if app.form.is_date_picker_open(&focused_key) {
                            if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                                if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                    let input_height = lr.size.height;
                                    let cal_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "calendar-picker");
                                    let dw = match cal_style.width {
                                        crate::style::Length::Px(v) => v,
                                        _ => 220.0,
                                    };
                                    let dh_total = match cal_style.height {
                                        crate::style::Length::Px(v) => v,
                                        _ => 210.0,
                                    };
                                    if app.mouse_x >= sx && app.mouse_x < sx + dw && app.mouse_y >= sy + input_height && app.mouse_y < sy + input_height + dh_total {
                                        needs_redraw = true;
                                    }
                                }
                            }
                        }
                    } else if focused_node.tag_name() == Some("input") && focused_node.get_attribute("type") == Some("time") {
                        if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                            if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                let sw = lr.size.width;
                                let sh = lr.size.height;
                                dropdown_hover = handle_time_picker_hover(app, &focused_key, &focused_node, sx, sy, sw, sh);
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
                if let Some(focused_node) = dom::Node::find_node_by_key(&root, focused_key) {
                    if focused_node.tag_name() == Some("select") {
                        if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                            if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                let sw = lr.size.width;
                                let sh = lr.size.height;
                                scrolled_dropdown = handle_select_scroll(app, focused_key, &focused_node, sx, sy, sw, sh, delta);
                            }
                        }
                    } else if focused_node.tag_name() == Some("input") && focused_node.get_attribute("type") == Some("date") {
                        if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                            if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                let sw = lr.size.width;
                                let sh = lr.size.height;
                                scrolled_dropdown = handle_date_picker_scroll(app, focused_key, &focused_node, sx, sy, sw, sh, delta);
                            }
                        }
                    } else if focused_node.tag_name() == Some("input") && focused_node.get_attribute("type") == Some("time") {
                        if let Some((sx, sy)) = get_node_abs_pos(&root, dom::node_ptr(&focused_node), &app.layout, 0.0, -app.scroll_y) {
                            if let Some(lr) = app.layout.get(dom::node_ptr(&focused_node)) {
                                let sw = lr.size.width;
                                let sh = lr.size.height;
                                scrolled_dropdown = handle_time_picker_scroll(app, focused_key, &focused_node, sx, sy, sw, sh, delta);
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


fn handle_select_click(
    app: &mut App,
    focused_key: &str,
    focused_node: &std::rc::Rc<dom::Node>,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
) -> bool {
    let mut options = Vec::new();
    for child in &focused_node.children {
        if child.tag_name() == Some("option") {
            options.push(child.clone());
        }
    }
    
    let opt_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "option");
    let opt_h = match opt_style.height {
        crate::style::Length::Px(v) => v,
        _ => 30.0,
    };
    let max_dropdown_h = if let Some(style) = app.styles.get(&dom::node_ptr(focused_node)) {
        match style.max_height {
            crate::style::Length::Px(v) => v,
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
        if let Some(w) = &app.window { w.request_redraw(); }
        true
    } else {
        false
    }
}

fn handle_date_picker_click(
    app: &mut App,
    focused_key: &str,
    focused_node: &std::rc::Rc<dom::Node>,
    sx: f32,
    sy: f32,
) -> bool {
    if let Some(lr) = app.layout.get(dom::node_ptr(focused_node)) {
        let input_height = lr.size.height;
        let cal_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "calendar-picker");
        let dw = match cal_style.width {
            crate::style::Length::Px(v) => v,
            _ => 220.0,
        };
        let dh_total = match cal_style.height {
            crate::style::Length::Px(v) => v,
            _ => 210.0,
        };
        
        if app.mouse_x >= sx && app.mouse_x < sx + dw && app.mouse_y >= sy + input_height && app.mouse_y < sy + input_height + dh_total {
            let my = app.mouse_y - (sy + input_height);
            let mx = app.mouse_x - sx;
            
            let (mut m, mut y) = app.form.get_date_picker_month_year(focused_key);
            
            if my < 35.0 {
                // Click in header
                if mx < 35.0 {
                    // Clicked "<"
                    if m == 1 {
                        m = 12;
                        y -= 1;
                    } else {
                        m -= 1;
                    }
                    app.form.set_date_picker_month_year(focused_key, m, y);
                } else if mx >= dw - 35.0 {
                    // Clicked ">"
                    if m == 12 {
                        m = 1;
                        y += 1;
                    } else {
                        m += 1;
                    }
                    app.form.set_date_picker_month_year(focused_key, m, y);
                }
            } else if my >= 60.0 {
                // Click in days grid
                let grid_y = my - 60.0;
                let col = (mx / (dw / 7.0)) as u32;
                let row_h = (dh_total - 60.0) / 6.0;
                let row = (grid_y / row_h) as u32;
                
                let first_dow = crate::form::day_of_week(y, m, 1);
                let total_days = crate::form::days_in_month(y, m);
                
                let idx = row * 7 + col;
                if idx >= first_dow && idx < first_dow + total_days {
                    let day = idx - first_dow + 1;
                    let format = focused_node.get_attribute("format").unwrap_or("dd/MM/yyyy");
                    
                    let new_val = if format.to_lowercase() == "yyyy-mm-dd" {
                        format!("{:04}-{:02}-{:02}", y, m, day)
                    } else {
                        format!("{:02}/{:02}/{:04}", day, m, y)
                    };
                    
                    app.form.set_value(focused_key, new_val);
                    app.form.set_date_picker_open(focused_key, false);
                    
                    app.form.set_cursor(focused_key, 10);
                    app.form.clear_selection(focused_key);
                }
            }
            if let Some(w) = &app.window { w.request_redraw(); }
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn handle_time_picker_click(
    app: &mut App,
    focused_key: &str,
    focused_node: &std::rc::Rc<dom::Node>,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
) -> bool {
    let format = focused_node.get_attribute("format").unwrap_or("HH:mm");
    let cursor_pos = app.form.cursor(focused_key);
    let active_section = app.form.get_time_active_section(focused_key).unwrap_or_else(|| {
        get_time_section_and_bounds(format, cursor_pos).0
    });
    
    let options = crate::render::date_dropdown::generate_time_options(active_section);
    
    let opt_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "option");
    let opt_h = match opt_style.height {
        crate::style::Length::Px(v) => v,
        _ => 30.0,
    };
    let input_style = app.styles.get(&dom::node_ptr(focused_node)).cloned().unwrap_or_default();
    let max_dropdown_h = match input_style.max_height {
        crate::style::Length::Px(v) => v,
        _ => {
            let time_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "time-picker");
            match time_style.max_height {
                crate::style::Length::Px(v) => v,
                _ => 7.0 * opt_h,
            }
        }
    };
    let total_h = options.len() as f32 * opt_h;
    let dropdown_h = total_h.min(max_dropdown_h);
    let dropdown_scroll = app.form.get_dropdown_scroll_y(focused_key);
    
    if app.mouse_x >= sx && app.mouse_x < sx + sw && app.mouse_y >= sy + sh && app.mouse_y < sy + sh + dropdown_h {
        let clicked_idx = ((app.mouse_y + dropdown_scroll - (sy + sh)) / opt_h) as usize;
        if clicked_idx < options.len() {
            let selected_val = &options[clicked_idx];
            let current_val = app.form.get_value(focused_key).to_string();
            let initial_val = if current_val.is_empty() { format.to_string() } else { current_val };
            let updated_val = update_time_value_section(&initial_val, format, active_section, selected_val);
            app.form.set_value(focused_key, updated_val);
            
            // Move cursor and update active section
            let format_lower = format.to_lowercase();
            if format_lower == "hh:mm:ss" {
                match active_section {
                    TimeSection::Hour => {
                        app.form.set_cursor(focused_key, 3);
                        app.form.set_time_active_section(focused_key, TimeSection::Minute);
                        app.form.set_dropdown_scroll_y(focused_key, 0.0);
                    }
                    TimeSection::Minute => {
                        app.form.set_cursor(focused_key, 6);
                        app.form.set_time_active_section(focused_key, TimeSection::Second);
                        app.form.set_dropdown_scroll_y(focused_key, 0.0);
                    }
                    TimeSection::Second => {
                        app.form.set_cursor(focused_key, 8);
                        app.focus_node(None);
                    }
                }
            } else {
                match active_section {
                    TimeSection::Hour => {
                        app.form.set_cursor(focused_key, 3);
                        app.form.set_time_active_section(focused_key, TimeSection::Minute);
                        app.form.set_dropdown_scroll_y(focused_key, 0.0);
                    }
                    TimeSection::Minute => {
                        app.form.set_cursor(focused_key, 5);
                        app.focus_node(None);
                    }
                    TimeSection::Second => {}
                }
            }
        }
        if let Some(w) = &app.window { w.request_redraw(); }
        true
    } else {
        false
    }
}

fn handle_select_hover(
    app: &mut App,
    focused_key: &str,
    focused_node: &std::rc::Rc<dom::Node>,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
) -> Option<usize> {
    let mut options = Vec::new();
    for child in &focused_node.children {
        if child.tag_name() == Some("option") {
            options.push(child.clone());
        }
    }
    
    let opt_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "option");
    let opt_h = match opt_style.height {
        crate::style::Length::Px(v) => v,
        _ => 30.0,
    };
    let max_dropdown_h = if let Some(style) = app.styles.get(&dom::node_ptr(focused_node)) {
        match style.max_height {
            crate::style::Length::Px(v) => v,
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
            return Some(clicked_idx);
        }
    }
    None
}

fn handle_time_picker_hover(
    app: &mut App,
    focused_key: &str,
    focused_node: &std::rc::Rc<dom::Node>,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
) -> Option<usize> {
    let format = focused_node.get_attribute("format").unwrap_or("HH:mm");
    let cursor_pos = app.form.cursor(focused_key);
    let active_section = app.form.get_time_active_section(focused_key).unwrap_or_else(|| {
        get_time_section_and_bounds(format, cursor_pos).0
    });
    
    let options = crate::render::date_dropdown::generate_time_options(active_section);
    
    let opt_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "option");
    let opt_h = match opt_style.height {
        crate::style::Length::Px(v) => v,
        _ => 30.0,
    };
    let input_style = app.styles.get(&dom::node_ptr(focused_node)).cloned().unwrap_or_default();
    let max_dropdown_h = match input_style.max_height {
        crate::style::Length::Px(v) => v,
        _ => {
            let time_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "time-picker");
            match time_style.max_height {
                crate::style::Length::Px(v) => v,
                _ => 7.0 * opt_h,
            }
        }
    };
    let total_h = options.len() as f32 * opt_h;
    let dropdown_h = total_h.min(max_dropdown_h);
    let dropdown_scroll = app.form.get_dropdown_scroll_y(focused_key);
    
    if app.mouse_x >= sx && app.mouse_x < sx + sw && app.mouse_y >= sy + sh && app.mouse_y < sy + sh + dropdown_h {
        let clicked_idx = ((app.mouse_y + dropdown_scroll - (sy + sh)) / opt_h) as usize;
        if clicked_idx < options.len() {
            return Some(clicked_idx);
        }
    }
    None
}

fn handle_select_scroll(
    app: &mut App,
    focused_key: &str,
    focused_node: &std::rc::Rc<dom::Node>,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    delta: MouseScrollDelta,
) -> bool {
    let mut options = Vec::new();
    for child in &focused_node.children {
        if child.tag_name() == Some("option") {
            options.push(child.clone());
        }
    }
    
    let opt_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "option");
    let opt_h = match opt_style.height {
        crate::style::Length::Px(v) => v,
        _ => 30.0,
    };
    let max_dropdown_h = if let Some(style) = app.styles.get(&dom::node_ptr(focused_node)) {
        match style.max_height {
            crate::style::Length::Px(v) => v,
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
        if let Some(w) = &app.window { w.request_redraw(); }
        true
    } else {
        false
    }
}

fn handle_date_picker_scroll(
    app: &mut App,
    focused_key: &str,
    focused_node: &std::rc::Rc<dom::Node>,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    delta: MouseScrollDelta,
) -> bool {
    let format = focused_node.get_attribute("format").unwrap_or("dd/MM/yyyy");
    let cursor_pos = app.form.cursor(focused_key);
    let active_section = app.form.get_date_active_section(focused_key).unwrap_or_else(|| {
        get_date_section_and_bounds(format, cursor_pos).0
    });
    
    let years_range = app.form.get_date_years_range(focused_key);
    let options = crate::render::date_dropdown::generate_date_options(active_section, years_range);
    
    let opt_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "option");
    let opt_h = match opt_style.height {
        crate::style::Length::Px(v) => v,
        _ => 30.0,
    };
    let input_style = app.styles.get(&dom::node_ptr(focused_node)).cloned().unwrap_or_default();
    let max_dropdown_h = match input_style.max_height {
        crate::style::Length::Px(v) => v,
        _ => 7.0 * opt_h,
    };
    let total_h = options.len() as f32 * opt_h;
    let dropdown_h = total_h.min(max_dropdown_h);
    
    if app.mouse_x >= sx && app.mouse_x < sx + sw && app.mouse_y >= sy + sh && app.mouse_y < sy + sh + dropdown_h {
        let scroll_y = app.form.get_dropdown_scroll_y(focused_key);
        let dy = match delta {
            MouseScrollDelta::LineDelta(_, y) => -y * 20.0,
            MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
        };
        
        if active_section == DateSection::Year {
            let (mut min_year, mut max_year) = years_range;
            let mut new_scroll = scroll_y + dy;
            
            if new_scroll < 20.0 {
                min_year -= 10;
                app.form.set_date_years_range(focused_key, (min_year, max_year));
                new_scroll += 10.0 * opt_h;
            } else {
                let total_h = ((max_year - min_year + 1) as f32) * opt_h;
                let max_scroll = (total_h - dropdown_h).max(0.0);
                if new_scroll > max_scroll - 20.0 {
                    max_year += 10;
                    app.form.set_date_years_range(focused_key, (min_year, max_year));
                }
            }
            
            let total_h = ((max_year - min_year + 1) as f32) * opt_h;
            let max_scroll = (total_h - dropdown_h).max(0.0);
            let clamped_scroll = new_scroll.clamp(0.0, max_scroll);
            app.form.set_dropdown_scroll_y(focused_key, clamped_scroll);
        } else {
            let max_scroll = (total_h - dropdown_h).max(0.0);
            let clamped_scroll = (scroll_y + dy).clamp(0.0, max_scroll);
            app.form.set_dropdown_scroll_y(focused_key, clamped_scroll);
        }
        
        if let Some(w) = &app.window { w.request_redraw(); }
        true
    } else {
        false
    }
}

fn handle_time_picker_scroll(
    app: &mut App,
    focused_key: &str,
    focused_node: &std::rc::Rc<dom::Node>,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    delta: MouseScrollDelta,
) -> bool {
    let format = focused_node.get_attribute("format").unwrap_or("HH:mm");
    let cursor_pos = app.form.cursor(focused_key);
    let active_section = app.form.get_time_active_section(focused_key).unwrap_or_else(|| {
        get_time_section_and_bounds(format, cursor_pos).0
    });
    
    let options = crate::render::date_dropdown::generate_time_options(active_section);
    
    let opt_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "option");
    let opt_h = match opt_style.height {
        crate::style::Length::Px(v) => v,
        _ => 30.0,
    };
    let input_style = app.styles.get(&dom::node_ptr(focused_node)).cloned().unwrap_or_default();
    let max_dropdown_h = match input_style.max_height {
        crate::style::Length::Px(v) => v,
        _ => {
            let time_style = crate::style::resolve_virtual_style(app.stylesheet.as_ref(), "time-picker");
            match time_style.max_height {
                crate::style::Length::Px(v) => v,
                _ => 7.0 * opt_h,
            }
        }
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
        let clamped_scroll = (scroll_y + dy).clamp(0.0, max_scroll);
        app.form.set_dropdown_scroll_y(focused_key, clamped_scroll);
        
        if let Some(w) = &app.window { w.request_redraw(); }
        true
    } else {
        false
    }
}
