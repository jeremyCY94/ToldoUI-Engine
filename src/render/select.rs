use std::collections::HashMap;
use std::rc::Rc;
use raqote::*;
use rusttype::Font;
use crate::dom::{self, Node};
use crate::form::FormState;
use crate::layout::LayoutEngine;
use crate::style::{ComputedStyle, StyleMap};
use super::primitives::lp;
use super::text::render_single_line_text;

pub fn paint_select(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    style: &ComputedStyle,
    form: &FormState,
    key: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let val = form.get_value(key);
    let arrow = "\u{25BC}";
    let aw2 = style.font_size * 0.8;
    if !val.is_empty() {
        let tx = x + lp(&style.padding_left) + style.border.left.width;
        let ty = y + (h - style.font_size) * 0.5;
        let txt_col = SolidSource::from_unpremultiplied_argb(200, 41, 41, 41);
        let mw2 = w - lp(&style.padding_left) - lp(&style.padding_right) - style.border.left.width - style.border.right.width - aw2 - 8.0;
        render_single_line_text(dt, fonts, style, val, tx, ty, mw2.max(1.0), txt_col, 0.0);
    }
    let ax = x + w - lp(&style.padding_right) - style.border.right.width - aw2 - 4.0;
    let ay = y + (h - style.font_size) * 0.5;
    let ac = SolidSource::from_unpremultiplied_argb(180, 80, 80, 80);
    render_single_line_text(dt, fonts, style, arrow, ax, ay, aw2 + 8.0, ac, 0.0);
}

pub fn paint_select_dropdown_overlay(
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
            if focused_node.tag_name() == Some("select") {
                let select_pos = get_node_abs_pos(root, dom::node_ptr(&focused_node), layout, 0.0, -scroll_y);
                if let Some((sx, sy)) = select_pos {
                    if let Some(lr) = layout.get(dom::node_ptr(&focused_node)) {
                        let sw = lr.size.width;
                        let sh = lr.size.height;
                        
                        let mut options = Vec::new();
                        for child in &focused_node.children {
                            if child.tag_name() == Some("option") {
                                options.push(child.clone());
                            }
                        }
                        
                        if !options.is_empty() {
                            let opt_h = 30.0;
                            let select_style = styles.get(&dom::node_ptr(&focused_node)).cloned().unwrap_or_default();
                            let max_dropdown_h = match select_style.max_height {
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
                            
                            // Clip content to dropdown viewport (leaving space for top/bottom borders)
                            let cx1 = sx as i32;
                            let cy1 = (sy + sh + 1.0) as i32;
                            let cx2 = (sx + sw) as i32;
                            let cy2 = (sy + sh + dropdown_h - 1.0) as i32;
                            dt.push_clip_rect(IntRect::new(IntPoint::new(cx1, cy1), IntPoint::new(cx2, cy2)));
                            
                            for (idx, option_node) in options.iter().enumerate() {
                                let opt_y = sy + sh + idx as f32 * opt_h - dropdown_scroll;
                                let opt_text = option_node.children_text().trim().to_string();
                                let opt_key = format!("{:p}", dom::node_ptr(option_node));
                                
                                // Only detect hover if mouse is within dropdown y-bounds
                                let is_hovered = mouse_x >= sx && mouse_x < sx + sw
                                    && mouse_y >= opt_y && mouse_y < opt_y + opt_h
                                    && mouse_y >= sy + sh && mouse_y < sy + sh + dropdown_h;
                                let is_selected = form.is_checked(&opt_key);
                                
                                if is_hovered {
                                    let hover_bg = SolidSource::from_unpremultiplied_argb(255, 50, 130, 250);
                                    dt.fill_rect(sx + 1.0, opt_y + 1.0, sw - 2.0, opt_h - 2.0, &Source::Solid(hover_bg), &DrawOptions::new());
                                } else if is_selected {
                                    let selected_bg = SolidSource::from_unpremultiplied_argb(255, 240, 240, 240);
                                    dt.fill_rect(sx + 1.0, opt_y + 1.0, sw - 2.0, opt_h - 2.0, &Source::Solid(selected_bg), &DrawOptions::new());
                                }
                                
                                let select_style = styles.get(&dom::node_ptr(&focused_node)).cloned().unwrap_or_default();
                                let mut text_style = select_style.clone();
                                text_style.font_size = 14.0;
                                
                                let text_color = if is_hovered {
                                    SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)
                                } else {
                                    SolidSource::from_unpremultiplied_argb(255, 41, 41, 41)
                                };
                                
                                let tx = sx + 10.0;
                                let ty = opt_y + (opt_h - text_style.font_size) * 0.5;
                                let mw2 = sw - 20.0;
                                
                                render_single_line_text(dt, fonts, &text_style, &opt_text, tx, ty, mw2.max(1.0), text_color, 0.0);
                                
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

pub fn get_node_abs_pos(
    root: &Rc<Node>,
    target_ptr: *const Node,
    layout: &LayoutEngine,
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

pub fn find_node_by_key(node: &Rc<Node>, target_key: &str) -> Option<Rc<Node>> {
    let key = format!("{:p}", dom::node_ptr(node));
    if key == target_key {
        return Some(node.clone());
    }
    for child in &node.children {
        if let Some(n) = find_node_by_key(child, target_key) {
            return Some(n);
        }
    }
    None
}
