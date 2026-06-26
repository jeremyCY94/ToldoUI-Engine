use std::collections::HashMap;
use std::rc::Rc;
use raqote::*;
use rusttype::Font;
use crate::dom::{self, Node};
use crate::form::{FormState, DateSection, TimeSection};
use crate::layout::LayoutEngine;
use crate::style::StyleMap;
use super::text::render_single_line_text;
use super::select::{get_node_abs_pos, find_node_by_key};

pub fn generate_date_options(section: DateSection, years_range: (i32, i32)) -> Vec<String> {
    match section {
        DateSection::Day => {
            (1..=31).map(|d| format!("{:02}", d)).collect()
        }
        DateSection::Month => {
            (1..=12).map(|m| format!("{:02}", m)).collect()
        }
        DateSection::Year => {
            (years_range.0..=years_range.1).map(|y| y.to_string()).collect()
        }
    }
}

pub fn get_date_section_value(val: &str, format: &str, section: DateSection) -> Option<String> {
    let format_lower = format.to_lowercase();
    let chars: Vec<char> = val.chars().collect();
    if format_lower == "yyyy-mm-dd" {
        if chars.len() < 10 {
            return None;
        }
        match section {
            DateSection::Year => Some(chars[0..4].iter().collect()),
            DateSection::Month => Some(chars[5..7].iter().collect()),
            DateSection::Day => Some(chars[8..10].iter().collect()),
        }
    } else {
        // dd/MM/yyyy
        if chars.len() < 10 {
            return None;
        }
        match section {
            DateSection::Day => Some(chars[0..2].iter().collect()),
            DateSection::Month => Some(chars[3..5].iter().collect()),
            DateSection::Year => Some(chars[6..10].iter().collect()),
        }
    }
}

pub fn paint_date_dropdown_overlay(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    styles: &StyleMap,
    layout: &LayoutEngine,
    form: &FormState,
    root: &Rc<Node>,
    scroll_y: f32,
    mouse_x: f32,
    mouse_y: f32,
) {
    if let Some(ref focused_key) = form.focused {
        if let Some(focused_node) = find_node_by_key(root, focused_key) {
            let is_date = focused_node.tag_name() == Some("input") && focused_node.get_attribute("type") == Some("date");
            if is_date {
                let format = focused_node.get_attribute("format").unwrap_or("dd/MM/yyyy");
                let val = form.get_value(focused_key);
                let cursor_pos = form.cursor(focused_key);
                
                // Determine active section (if not explicitly set, calculate it using helper)
                let active_section = form.get_date_active_section(focused_key).unwrap_or_else(|| {
                    // Fallback to cursor position check
                    let format_lower = format.to_lowercase();
                    if format_lower == "yyyy-mm-dd" {
                        if cursor_pos < 4 {
                            DateSection::Year
                        } else if cursor_pos >= 5 && cursor_pos < 7 {
                            DateSection::Month
                        } else if cursor_pos >= 8 {
                            DateSection::Day
                        } else {
                            if cursor_pos == 4 { DateSection::Year } else { DateSection::Month }
                        }
                    } else {
                        if cursor_pos < 2 {
                            DateSection::Day
                        } else if cursor_pos >= 3 && cursor_pos < 5 {
                            DateSection::Month
                        } else if cursor_pos >= 6 {
                            DateSection::Year
                        } else {
                            if cursor_pos == 2 { DateSection::Day } else { DateSection::Month }
                        }
                    }
                });
                
                let years_range = form.get_date_years_range(focused_key);
                let options = generate_date_options(active_section, years_range);
                
                let select_pos = get_node_abs_pos(root, dom::node_ptr(&focused_node), layout, 0.0, -scroll_y);
                if let Some((sx, sy)) = select_pos {
                    if let Some(lr) = layout.get(dom::node_ptr(&focused_node)) {
                        let sw = lr.size.width;
                        let sh = lr.size.height;
                        
                        if !options.is_empty() {
                            let opt_h = 30.0;
                            let input_style = styles.get(&dom::node_ptr(&focused_node)).cloned().unwrap_or_default();
                            let max_dropdown_h = match input_style.max_height {
                                crate::style::Length::Px(v) => v,
                                _ => 7.0 * opt_h,
                            };
                            let total_h = options.len() as f32 * opt_h;
                            let dropdown_h = total_h.min(max_dropdown_h);
                            let dropdown_scroll = form.get_dropdown_scroll_y(focused_key);
                            
                            // Draw shadow
                            let shadow_col = SolidSource::from_unpremultiplied_argb(35, 0, 0, 0);
                            dt.fill_rect(sx + 3.0, sy + sh + 3.0, sw, dropdown_h, &Source::Solid(shadow_col), &DrawOptions::new());
                            
                            // Main background (White)
                            let bg_color = SolidSource::from_unpremultiplied_argb(255, 255, 255, 255);
                            dt.fill_rect(sx, sy + sh, sw, dropdown_h, &Source::Solid(bg_color), &DrawOptions::new());
                            
                            // Clip content to dropdown viewport
                            let cx1 = sx as i32;
                            let cy1 = (sy + sh + 1.0) as i32;
                            let cx2 = (sx + sw) as i32;
                            let cy2 = (sy + sh + dropdown_h - 1.0) as i32;
                            dt.push_clip_rect(IntRect::new(IntPoint::new(cx1, cy1), IntPoint::new(cx2, cy2)));
                            
                            // Find currently selected value for active section
                            let current_selected_val = get_date_section_value(&val, format, active_section);
                            
                            for (idx, opt_text) in options.iter().enumerate() {
                                let opt_y = sy + sh + idx as f32 * opt_h - dropdown_scroll;
                                
                                // Only detect hover if mouse is within dropdown bounds
                                let is_hovered = mouse_x >= sx && mouse_x < sx + sw
                                    && mouse_y >= opt_y && mouse_y < opt_y + opt_h
                                    && mouse_y >= sy + sh && mouse_y < sy + sh + dropdown_h;
                                let is_selected = current_selected_val.as_deref() == Some(opt_text);
                                
                                if is_hovered {
                                    let hover_bg = SolidSource::from_unpremultiplied_argb(255, 50, 130, 250);
                                    dt.fill_rect(sx + 1.0, opt_y + 1.0, sw - 2.0, opt_h - 2.0, &Source::Solid(hover_bg), &DrawOptions::new());
                                } else if is_selected {
                                    let selected_bg = SolidSource::from_unpremultiplied_argb(255, 240, 240, 240);
                                    dt.fill_rect(sx + 1.0, opt_y + 1.0, sw - 2.0, opt_h - 2.0, &Source::Solid(selected_bg), &DrawOptions::new());
                                }
                                
                                let mut text_style = input_style.clone();
                                text_style.font_size = 14.0;
                                
                                let text_color = if is_hovered {
                                    SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)
                                } else {
                                    SolidSource::from_unpremultiplied_argb(255, 41, 41, 41)
                                };
                                
                                let tx = sx + 10.0;
                                let ty = opt_y + (opt_h - text_style.font_size) * 0.5;
                                let mw2 = sw - 20.0;
                                
                                render_single_line_text(dt, fonts, &text_style, opt_text, tx, ty, mw2.max(1.0), text_color, 0.0);
                                
                                if idx < options.len() - 1 {
                                    let sep_color = SolidSource::from_unpremultiplied_argb(255, 240, 240, 240);
                                    dt.fill_rect(sx + 1.0, opt_y + opt_h - 1.0, sw - 2.0, 1.0, &Source::Solid(sep_color), &DrawOptions::new());
                                }
                            }
                            
                            dt.pop_clip();
                            
                            // Draw scrollbar if content exceeds visible area
                            if total_h > dropdown_h {
                                let sb_w = 6.0;
                                let sb_x = sx + sw - sb_w - 2.0;
                                let track_h = dropdown_h - 4.0;
                                let thumb_h = (dropdown_h / total_h * track_h).max(15.0);
                                let thumb_y = sy + sh + 2.0 + (dropdown_scroll / (total_h - dropdown_h)) * (track_h - thumb_h);
                                
                                // Track background
                                let track_col = SolidSource::from_unpremultiplied_argb(20, 0, 0, 0);
                                dt.fill_rect(sb_x, sy + sh + 2.0, sb_w, track_h, &Source::Solid(track_col), &DrawOptions::new());
                                
                                // Thumb
                                let thumb_col = SolidSource::from_unpremultiplied_argb(100, 0, 0, 0);
                                dt.fill_rect(sb_x, thumb_y, sb_w, thumb_h, &Source::Solid(thumb_col), &DrawOptions::new());
                            }
                            
                            // Borders (Drawn on top)
                            let border_color = SolidSource::from_unpremultiplied_argb(255, 200, 200, 200);
                            dt.fill_rect(sx, sy + sh, sw, 1.0, &Source::Solid(border_color), &DrawOptions::new());
                            dt.fill_rect(sx, sy + sh + dropdown_h, sw, 1.0, &Source::Solid(border_color), &DrawOptions::new());
                            dt.fill_rect(sx, sy + sh, 1.0, dropdown_h, &Source::Solid(border_color), &DrawOptions::new());
                            dt.fill_rect(sx + sw - 1.0, sy + sh, 1.0, dropdown_h, &Source::Solid(border_color), &DrawOptions::new());
                        }
                    }
                }
            }
        }
    }
}

pub fn generate_time_options(section: TimeSection) -> Vec<String> {
    match section {
        TimeSection::Hour => {
            (0..=23).map(|h| format!("{:02}", h)).collect()
        }
        TimeSection::Minute => {
            (0..=59).map(|m| format!("{:02}", m)).collect()
        }
        TimeSection::Second => {
            (0..=59).map(|s| format!("{:02}", s)).collect()
        }
    }
}

pub fn get_time_section_value(val: &str, format: &str, section: TimeSection) -> Option<String> {
    let chars: Vec<char> = val.chars().collect();
    let format_lower = format.to_lowercase();
    if format_lower == "hh:mm:ss" {
        if chars.len() < 8 {
            return None;
        }
        match section {
            TimeSection::Hour => Some(chars[0..2].iter().collect()),
            TimeSection::Minute => Some(chars[3..5].iter().collect()),
            TimeSection::Second => Some(chars[6..8].iter().collect()),
        }
    } else {
        if chars.len() < 5 {
            return None;
        }
        match section {
            TimeSection::Hour => Some(chars[0..2].iter().collect()),
            TimeSection::Minute => Some(chars[3..5].iter().collect()),
            TimeSection::Second => None,
        }
    }
}

pub fn paint_time_dropdown_overlay(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    styles: &StyleMap,
    layout: &LayoutEngine,
    form: &FormState,
    root: &Rc<Node>,
    scroll_y: f32,
    mouse_x: f32,
    mouse_y: f32,
) {
    if let Some(ref focused_key) = form.focused {
        if let Some(focused_node) = find_node_by_key(root, focused_key) {
            let is_time = focused_node.tag_name() == Some("input") && focused_node.get_attribute("type") == Some("time");
            if is_time {
                let format = focused_node.get_attribute("format").unwrap_or("HH:mm");
                let val = form.get_value(focused_key);
                let cursor_pos = form.cursor(focused_key);
                
                let active_section = form.get_time_active_section(focused_key).unwrap_or_else(|| {
                    let format_lower = format.to_lowercase();
                    if format_lower == "hh:mm:ss" {
                        if cursor_pos < 2 {
                            TimeSection::Hour
                        } else if cursor_pos >= 3 && cursor_pos < 5 {
                            TimeSection::Minute
                        } else {
                            TimeSection::Second
                        }
                    } else {
                        if cursor_pos < 2 {
                            TimeSection::Hour
                        } else {
                            TimeSection::Minute
                        }
                    }
                });
                
                let options = generate_time_options(active_section);
                
                let select_pos = get_node_abs_pos(root, dom::node_ptr(&focused_node), layout, 0.0, -scroll_y);
                if let Some((sx, sy)) = select_pos {
                    if let Some(lr) = layout.get(dom::node_ptr(&focused_node)) {
                        let sw = lr.size.width;
                        let sh = lr.size.height;
                        
                        if !options.is_empty() {
                            let opt_h = 30.0;
                            let input_style = styles.get(&dom::node_ptr(&focused_node)).cloned().unwrap_or_default();
                            let max_dropdown_h = match input_style.max_height {
                                crate::style::Length::Px(v) => v,
                                _ => 7.0 * opt_h,
                            };
                            let total_h = options.len() as f32 * opt_h;
                            let dropdown_h = total_h.min(max_dropdown_h);
                            let dropdown_scroll = form.get_dropdown_scroll_y(focused_key);
                            
                            // Draw shadow
                            let shadow_col = SolidSource::from_unpremultiplied_argb(35, 0, 0, 0);
                            dt.fill_rect(sx + 3.0, sy + sh + 3.0, sw, dropdown_h, &Source::Solid(shadow_col), &DrawOptions::new());
                            
                            // Main background (White)
                            let bg_color = SolidSource::from_unpremultiplied_argb(255, 255, 255, 255);
                            dt.fill_rect(sx, sy + sh, sw, dropdown_h, &Source::Solid(bg_color), &DrawOptions::new());
                            
                            // Clip content to dropdown viewport
                            let cx1 = sx as i32;
                            let cy1 = (sy + sh + 1.0) as i32;
                            let cx2 = (sx + sw) as i32;
                            let cy2 = (sy + sh + dropdown_h - 1.0) as i32;
                            dt.push_clip_rect(IntRect::new(IntPoint::new(cx1, cy1), IntPoint::new(cx2, cy2)));
                            
                            let current_selected_val = get_time_section_value(&val, format, active_section);
                            
                            for (idx, opt_text) in options.iter().enumerate() {
                                let opt_y = sy + sh + idx as f32 * opt_h - dropdown_scroll;
                                
                                let is_hovered = mouse_x >= sx && mouse_x < sx + sw
                                    && mouse_y >= opt_y && mouse_y < opt_y + opt_h
                                    && mouse_y >= sy + sh && mouse_y < sy + sh + dropdown_h;
                                let is_selected = current_selected_val.as_deref() == Some(opt_text);
                                
                                if is_hovered {
                                    let hover_bg = SolidSource::from_unpremultiplied_argb(255, 50, 130, 250);
                                    dt.fill_rect(sx + 1.0, opt_y + 1.0, sw - 2.0, opt_h - 2.0, &Source::Solid(hover_bg), &DrawOptions::new());
                                } else if is_selected {
                                    let selected_bg = SolidSource::from_unpremultiplied_argb(255, 240, 240, 240);
                                    dt.fill_rect(sx + 1.0, opt_y + 1.0, sw - 2.0, opt_h - 2.0, &Source::Solid(selected_bg), &DrawOptions::new());
                                }
                                
                                let mut text_style = input_style.clone();
                                text_style.font_size = 14.0;
                                
                                let text_color = if is_hovered {
                                    SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)
                                } else {
                                    SolidSource::from_unpremultiplied_argb(255, 41, 41, 41)
                                };
                                
                                let tx = sx + 10.0;
                                let ty = opt_y + (opt_h - text_style.font_size) * 0.5;
                                let mw2 = sw - 20.0;
                                
                                render_single_line_text(dt, fonts, &text_style, opt_text, tx, ty, mw2.max(1.0), text_color, 0.0);
                                
                                if idx < options.len() - 1 {
                                    let sep_color = SolidSource::from_unpremultiplied_argb(255, 240, 240, 240);
                                    dt.fill_rect(sx + 1.0, opt_y + opt_h - 1.0, sw - 2.0, 1.0, &Source::Solid(sep_color), &DrawOptions::new());
                                }
                            }
                            
                            dt.pop_clip();
                            
                            // Draw scrollbar if content exceeds visible area
                            if total_h > dropdown_h {
                                let sb_w = 6.0;
                                let sb_x = sx + sw - sb_w - 2.0;
                                let track_h = dropdown_h - 4.0;
                                let thumb_h = (dropdown_h / total_h * track_h).max(15.0);
                                let thumb_y = sy + sh + 2.0 + (dropdown_scroll / (total_h - dropdown_h)) * (track_h - thumb_h);
                                
                                let track_col = SolidSource::from_unpremultiplied_argb(20, 0, 0, 0);
                                dt.fill_rect(sb_x, sy + sh + 2.0, sb_w, track_h, &Source::Solid(track_col), &DrawOptions::new());
                                
                                let thumb_col = SolidSource::from_unpremultiplied_argb(100, 0, 0, 0);
                                dt.fill_rect(sb_x, thumb_y, sb_w, thumb_h, &Source::Solid(thumb_col), &DrawOptions::new());
                            }
                            
                            let border_color = SolidSource::from_unpremultiplied_argb(255, 200, 200, 200);
                            dt.fill_rect(sx, sy + sh, sw, 1.0, &Source::Solid(border_color), &DrawOptions::new());
                            dt.fill_rect(sx, sy + sh + dropdown_h, sw, 1.0, &Source::Solid(border_color), &DrawOptions::new());
                            dt.fill_rect(sx, sy + sh, 1.0, dropdown_h, &Source::Solid(border_color), &DrawOptions::new());
                            dt.fill_rect(sx + sw - 1.0, sy + sh, 1.0, dropdown_h, &Source::Solid(border_color), &DrawOptions::new());
                        }
                    }
                }
            }
        }
    }
}
