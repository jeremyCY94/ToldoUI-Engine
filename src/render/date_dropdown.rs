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

const MONTH_NAMES: [&str; 12] = [
    "Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio",
    "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre"
];

fn draw_circle_path(cx: f32, cy: f32, r: f32) -> Path {
    let mut pb = PathBuilder::new();
    let kappa = 0.55228475;
    let ox = r * kappa;
    
    pb.move_to(cx + r, cy);
    pb.cubic_to(cx + r, cy + ox, cx + ox, cy + r, cx, cy + r);
    pb.cubic_to(cx - ox, cy + r, cx - r, cy + ox, cx - r, cy);
    pb.cubic_to(cx - r, cy - ox, cx - ox, cy - r, cx, cy - r);
    pb.cubic_to(cx + ox, cy - r, cx + r, cy - ox, cx + r, cy);
    pb.close();
    pb.finish()
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
            if is_date && form.is_date_picker_open(focused_key) {
                let select_pos = get_node_abs_pos(root, dom::node_ptr(&focused_node), layout, 0.0, -scroll_y);
                if let Some((sx, sy)) = select_pos {
                    if let Some(lr) = layout.get(dom::node_ptr(&focused_node)) {
                        let input_height = lr.size.height;
                        let dw = 220.0f32; // calendar width
                        let dh_total = 210.0f32; // calendar height
                        let input_style = styles.get(&dom::node_ptr(&focused_node)).cloned().unwrap_or_default();
                        
                        // Draw shadow
                        let shadow_col = SolidSource::from_unpremultiplied_argb(35, 0, 0, 0);
                        dt.fill_rect(sx + 3.0, sy + input_height + 3.0, dw, dh_total, &Source::Solid(shadow_col), &DrawOptions::new());
                        
                        // Main background (White)
                        let bg_color = SolidSource::from_unpremultiplied_argb(255, 255, 255, 255);
                        dt.fill_rect(sx, sy + input_height, dw, dh_total, &Source::Solid(bg_color), &DrawOptions::new());
                        
                        // Main border
                        let border_color = SolidSource::from_unpremultiplied_argb(255, 210, 214, 219);
                        let stroke_style = StrokeStyle {
                            width: 1.0,
                            cap: LineCap::Butt,
                            join: LineJoin::Miter,
                            miter_limit: 10.0,
                            dash_array: Vec::new(),
                            dash_offset: 0.0,
                        };
                        let mut pb_border = PathBuilder::new();
                        pb_border.rect(sx, sy + input_height, dw, dh_total);
                        dt.stroke(&pb_border.finish(), &Source::Solid(border_color), &stroke_style, &DrawOptions::new());
                        
                        // Get date picker month and year
                        let (m, y) = form.get_date_picker_month_year(focused_key);
                        
                        // Render Header row: `<` (Left arrow), Month/Year text, `>` (Right arrow)
                        let header_y = sy + input_height;
                        let header_h = 35.0f32;
                        
                        // Left arrow "<" button hover check
                        let left_hover = mouse_x >= sx && mouse_x < sx + 35.0 && mouse_y >= header_y && mouse_y < header_y + header_h;
                        if left_hover {
                            let hover_bg = SolidSource::from_unpremultiplied_argb(255, 240, 242, 245);
                            dt.fill_rect(sx + 1.0, header_y + 1.0, 33.0, header_h - 2.0, &Source::Solid(hover_bg), &DrawOptions::new());
                        }
                        
                        // Right arrow ">" button hover check
                        let right_hover = mouse_x >= sx + dw - 35.0 && mouse_x < sx + dw && mouse_y >= header_y && mouse_y < header_y + header_h;
                        if right_hover {
                            let hover_bg = SolidSource::from_unpremultiplied_argb(255, 240, 242, 245);
                            dt.fill_rect(sx + dw - 34.0, header_y + 1.0, 33.0, header_h - 2.0, &Source::Solid(hover_bg), &DrawOptions::new());
                        }
                        
                        // Draw Header Texts
                        let mut text_style = input_style.clone();
                        text_style.font_size = 13.0;
                        
                        let arrow_color = SolidSource::from_unpremultiplied_argb(255, 80, 80, 80);
                        let label_color = SolidSource::from_unpremultiplied_argb(255, 40, 40, 40);
                        
                        // Left Arrow "<" text
                        render_single_line_text(dt, fonts, &text_style, "<", sx + 13.0, header_y + (header_h - text_style.font_size) * 0.5, 20.0, arrow_color, 0.0);
                        // Right Arrow ">" text
                        render_single_line_text(dt, fonts, &text_style, ">", sx + dw - 21.0, header_y + (header_h - text_style.font_size) * 0.5, 20.0, arrow_color, 0.0);
                        
                        // Month/Year central label
                        let month_str = MONTH_NAMES.get((m as usize).saturating_sub(1)).copied().unwrap_or("Marzo");
                        let header_label = format!("{} {}", month_str, y);
                        text_style.font_weight = 700; // Bold header label
                        
                        let label_w = header_label.chars().count() as f32 * 7.5;
                        let lx = sx + (dw - label_w) * 0.5;
                        render_single_line_text(dt, fonts, &text_style, &header_label, lx, header_y + (header_h - text_style.font_size) * 0.5, 150.0, label_color, 0.0);
                        
                        // Divider line under header
                        let sep_color = SolidSource::from_unpremultiplied_argb(255, 230, 233, 238);
                        dt.fill_rect(sx + 1.0, header_y + header_h - 1.0, dw - 2.0, 1.0, &Source::Solid(sep_color), &DrawOptions::new());
                        
                        // Render Weekday headers row
                        let dow_y = header_y + header_h;
                        let dow_h = 25.0f32;
                        let col_w = dw / 7.0;
                        
                        let day_headers = ["Do", "Lu", "Ma", "Mi", "Ju", "Vi", "Sá"];
                        let mut dow_style = input_style.clone();
                        dow_style.font_size = 11.0;
                        let dow_color = SolidSource::from_unpremultiplied_argb(255, 140, 145, 155);
                        
                        for (col_idx, &dow_name) in day_headers.iter().enumerate() {
                            let dx = sx + col_idx as f32 * col_w + (col_w - 14.0) * 0.5;
                            render_single_line_text(dt, fonts, &dow_style, dow_name, dx, dow_y + (dow_h - dow_style.font_size) * 0.5, col_w, dow_color, 0.0);
                        }
                        
                        // Divider line under weekday headers
                        dt.fill_rect(sx + 1.0, dow_y + dow_h - 1.0, dw - 2.0, 1.0, &Source::Solid(sep_color), &DrawOptions::new());
                        
                        // Render Days grid
                        let grid_y = dow_y + dow_h;
                        let row_h = 25.0f32;
                        
                        let first_dow = crate::form::day_of_week(y, m, 1);
                        let total_days = crate::form::days_in_month(y, m);
                        
                        // Parse current date to highlight it if it matches
                        let current_val = form.get_value(focused_key);
                        let format = focused_node.get_attribute("format").unwrap_or("dd/MM/yyyy");
                        let parsed_current_date = crate::form::parse_date_value(current_val, format);
                        
                        let mut day_style = input_style.clone();
                        day_style.font_size = 12.0;
                        
                        for row in 0..6 {
                            for col in 0..7 {
                                let cell_idx = row * 7 + col;
                                if cell_idx >= first_dow as usize && cell_idx < (first_dow + total_days) as usize {
                                    let day = cell_idx - first_dow as usize + 1;
                                    
                                    let cell_x1 = sx + col as f32 * col_w;
                                    let cell_y1 = grid_y + row as f32 * row_h;
                                    
                                    // Check hover/selection
                                    let is_hovered = mouse_x >= cell_x1 && mouse_x < cell_x1 + col_w
                                        && mouse_y >= cell_y1 && mouse_y < cell_y1 + row_h;
                                    
                                    let is_selected = if let Some((curr_d, curr_m, curr_y)) = parsed_current_date {
                                        day as u32 == curr_d && m == curr_m && y == curr_y
                                    } else {
                                        false
                                    };
                                    
                                    if is_selected {
                                        let circle_r = 10.0f32;
                                        let cx = cell_x1 + col_w * 0.5;
                                        let cy = cell_y1 + row_h * 0.5;
                                        let active_bg = SolidSource::from_unpremultiplied_argb(255, 50, 130, 250);
                                        let circle_path = draw_circle_path(cx, cy, circle_r);
                                        dt.fill(&circle_path, &Source::Solid(active_bg), &DrawOptions::new());
                                    } else if is_hovered {
                                        let circle_r = 10.0f32;
                                        let cx = cell_x1 + col_w * 0.5;
                                        let cy = cell_y1 + row_h * 0.5;
                                        let hover_bg = SolidSource::from_unpremultiplied_argb(255, 235, 238, 243);
                                        let circle_path = draw_circle_path(cx, cy, circle_r);
                                        dt.fill(&circle_path, &Source::Solid(hover_bg), &DrawOptions::new());
                                    }
                                    
                                    let text_color = if is_selected {
                                        SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)
                                    } else {
                                        SolidSource::from_unpremultiplied_argb(255, 40, 40, 40)
                                    };
                                    
                                    let day_text = day.to_string();
                                    let char_count = day_text.chars().count() as f32;
                                    let tx = cell_x1 + (col_w - char_count * 6.5) * 0.5;
                                    let ty = cell_y1 + (row_h - day_style.font_size) * 0.5;
                                    
                                    render_single_line_text(dt, fonts, &day_style, &day_text, tx, ty, col_w, text_color, 0.0);
                                }
                            }
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
