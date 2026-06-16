use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::{Key, NamedKey};
use winit::dpi::PhysicalPosition;

use toldo_ui_engine::dom;
use toldo_ui_engine::form::actions::{delete_selected_text, get_selected_text, insert_text};

use crate::core::app::{App, get_node_abs_pos};

pub(crate) fn handle_keyboard(app: &mut App, event: winit::event::KeyEvent) {
    if event.state == ElementState::Pressed {
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
            match &event.logical_key {
                Key::Named(NamedKey::ArrowUp) => {
                    app.scroll_y = (app.scroll_y - 30.0).max(0.0);
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Named(NamedKey::ArrowDown) => {
                    app.scroll_y += 30.0;
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Named(NamedKey::PageUp) => {
                    app.scroll_y = (app.scroll_y - 300.0).max(0.0);
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Named(NamedKey::PageDown) => {
                    app.scroll_y += 300.0;
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                Key::Character(c) if c == "r" || c == "R" => {
                    if let (Some(html), Some(css)) = (app.initial_html.clone(), app.initial_css.clone()) {
                        app.load(&html, &css);
                    }
                    if let Some(w) = &app.window { w.request_redraw(); }
                }
                _ => {}
            }
        }
    }
}

pub(crate) fn handle_mouse_input(app: &mut App, state: ElementState, button: MouseButton) {
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

                if let Some((node, form_type)) = app.hit_test(app.mouse_x, app.mouse_y + app.scroll_y) {
                    let key = format!("{:p}", dom::node_ptr(&node));
                    match form_type {
                        "checkbox" => { app.form.toggle(&key); app.focus_node(None); app.click_count = 0; }
                        "radio" => { app.form.toggle(&key); app.focus_node(None); app.click_count = 0; }
                        "text" | "textarea" => {
                            app.focus_node(Some(key.clone()));
                            let val = app.form.get_value(&key);
                            let mut start_idx = val.chars().count();
                            let node_ptr = dom::node_ptr(&node);
                            if let Some(root) = app.dom.as_ref().and_then(|d| d.document_element()) {
                                if let Some((node_x, _)) = get_node_abs_pos(&root, node_ptr, &app.layout, 0.0, 0.0) {
                                    if let Some(style) = app.styles.get(&node_ptr) {
                                        let padding_left = match style.padding_left { toldo_ui_engine::style::Length::Px(v) => v, _ => 0.0 };
                                        let border_left = style.border.left.width;
                                        let cx = node_x + padding_left + border_left;
                                        let target_x = app.mouse_x - cx;
                                        start_idx = toldo_ui_engine::render::painter::index_at_x(style, val, target_x);
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
                        app.scroll_y = (ratio * max_scroll).clamp(0.0, max_scroll);
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

    if app.dragging_scrollbar {
        let win = match &app.window { Some(w) => w.clone(), None => return };
        let size = win.inner_size();
        let wh = size.height.max(100) as f32;
        let ch = app.dom.as_ref().and_then(|d| d.document_element()).map(|r| app.layout.content_height(&r)).unwrap_or(0.0);
        let max_scroll = (ch - wh).max(0.0);
        let track_h = wh - 4.0;
        let thumb_h = (wh / ch * track_h).max(20.0);
        let ratio = (app.mouse_y - 2.0 - thumb_h * 0.5) / (track_h - thumb_h);
        app.scroll_y = (ratio * max_scroll).clamp(0.0, max_scroll);
        if let Some(w) = &app.window { w.request_redraw(); }
    }

    if app.dragging_select {
        if let Some(node_ptr) = app.dragging_node {
            let key = format!("{:p}", node_ptr);
            if let Some(root) = app.dom.as_ref().and_then(|d| d.document_element()) {
                if let Some((node_x, _)) = get_node_abs_pos(&root, node_ptr, &app.layout, 0.0, 0.0) {
                    if let Some(style) = app.styles.get(&node_ptr) {
                        let padding_left = match style.padding_left { toldo_ui_engine::style::Length::Px(v) => v, _ => 0.0 };
                        let border_left = style.border.left.width;
                        let cx = node_x + padding_left + border_left;
                        let val = app.form.get_value(&key);
                        let target_x = app.mouse_x - cx;
                        let current_idx = toldo_ui_engine::render::painter::index_at_x(style, val, target_x);

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
    if app.update_hover() {
        if let Some(w) = &app.window { w.request_redraw(); }
    }
}

pub(crate) fn handle_mouse_wheel(app: &mut App, delta: MouseScrollDelta) {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => {
            app.scroll_y = (app.scroll_y - y * 20.0).max(0.0);
        }
        MouseScrollDelta::PixelDelta(pos) => {
            app.scroll_y = (app.scroll_y - pos.y as f32).max(0.0);
        }
    }
    app.dragging_scrollbar = false;
    app.update_cursor_icon();
    app.update_hover();
    if let Some(w) = &app.window { w.request_redraw(); }
}
