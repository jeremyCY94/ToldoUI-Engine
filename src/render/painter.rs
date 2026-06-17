use std::collections::HashMap;
use std::rc::Rc;

use raqote::*;
use rusttype::{Font, Scale, point as rpoint};

use crate::dom::{self, Node, NodeType};
use crate::form::FormState;
use crate::layout::LayoutEngine;
use crate::style::{BorderSide, BorderLineStyle, ComputedStyle, StyleMap, TextAlign};

pub struct Painter {
    fonts: HashMap<String, Font<'static>>,
}

impl Painter {
    pub fn new() -> Self { Painter { fonts: HashMap::new() } }

    fn load_font(&mut self, family: &str, weight: u16) -> Option<Font<'static>> {
        let key = format!("{}-{}", family.to_lowercase(), weight);
        if let Some(f) = self.fonts.get(&key) { return Some(f.clone()); }
        let data = load_font_data(family, weight)?;
        let font = Font::try_from_vec(data)?;
        self.fonts.insert(key, font.clone());
        Some(font)
    }

    pub fn paint(&mut self, dt: &mut DrawTarget, styles: &StyleMap, layout: &LayoutEngine, form: &FormState, root: Rc<Node>, scroll_y: f32, content_h: f32, vw: f32, vh: f32, caret_on: bool, mouse_x: f32, mouse_y: f32) {
        dt.clear(SolidSource::from_unpremultiplied_argb(255, 255, 255, 255));
        self.paint_node(dt, styles, layout, form, &root, 0.0, -scroll_y, caret_on);
        if content_h > vh {
            let sb_w = 10.0;
            let sb_x = vw - sb_w - 2.0;
            let track_h = vh - 4.0;
            let thumb_h = (vh / content_h * track_h).max(20.0);
            let thumb_y = 2.0 + (scroll_y / (content_h - vh)) * (track_h - thumb_h);
            let src = SolidSource::from_unpremultiplied_argb(40, 0, 0, 0);
            dt.fill_rect(sb_x, 2.0, sb_w, track_h, &Source::Solid(src), &DrawOptions::new());
            let src2 = SolidSource::from_unpremultiplied_argb(120, 0, 0, 0);
            dt.fill_rect(sb_x, thumb_y, sb_w, thumb_h, &Source::Solid(src2), &DrawOptions::new());
        }

        // Draw the select dropdown overlay if a select is focused
        if let Some(ref focused_key) = form.focused {
            if let Some(focused_node) = find_node_by_key(&root, focused_key) {
                if focused_node.tag_name() == Some("select") {
                    let select_pos = get_node_abs_pos(&root, dom::node_ptr(&focused_node), layout, 0.0, -scroll_y);
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
                                    
                                    self.render_single_line_text(dt, &text_style, &opt_text, tx, ty, mw2.max(1.0), text_color, 0.0);
                                    
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

    fn paint_node(&mut self, dt: &mut DrawTarget, styles: &StyleMap, layout: &LayoutEngine, form: &FormState, node: &Rc<Node>, px: f32, py: f32, caret_on: bool) {
        let ptr = dom::node_ptr(node);
        match &node.node_type {
            NodeType::Document => { for c in &node.children { self.paint_node(dt, styles, layout, form, c, px, py, caret_on); } }
            NodeType::Element(_) => {
                let style = match styles.get(&ptr) { Some(s) => s.clone(), None => { for c in &node.children { self.paint_node(dt, styles, layout, form, c, px, py, caret_on); } return; } };
                if style.display == taffy::Display::None || !style.visibility { return; }

                if let Some(lr) = layout.get(ptr) {
                    let x = px + lr.location.x;
                    let y = py + lr.location.y;
                    let w = lr.size.width;
                    let h = lr.size.height;

                    bg(dt, &style, x, y, w, h);
                    border(dt, &style, x, y, w, h);

                    let tag = node.tag_name().unwrap_or("");
                    let key = format!("{:p}", ptr);

                    match tag {
                        "input" => {
                            let itype = node.get_attribute("type").unwrap_or("text");
                            let cx = x + lp(&style.padding_left) + style.border.left.width;
                            match itype {
                                "checkbox" => {
                                    let cb_size = h.min(18.0);
                                    let cbx = x + (w - cb_size) * 0.5;
                                    let cby = y + (h - cb_size) * 0.5;
                                    let bs = SolidSource::from_unpremultiplied_argb(200, 80,80,80);
                                    dt.fill_rect(cbx, cby, cb_size, cb_size, &Source::Solid(SolidSource::from_unpremultiplied_argb(255,255,255,255)), &DrawOptions::new());
                                    dt.fill_rect(cbx, cby, cb_size, 1.0, &Source::Solid(bs), &DrawOptions::new());
                                    dt.fill_rect(cbx, cby+cb_size-1.0, cb_size, 1.0, &Source::Solid(bs), &DrawOptions::new());
                                    dt.fill_rect(cbx, cby, 1.0, cb_size, &Source::Solid(bs), &DrawOptions::new());
                                    dt.fill_rect(cbx+cb_size-1.0, cby, 1.0, cb_size, &Source::Solid(bs), &DrawOptions::new());
                                    if form.is_checked(&key) {
                                        let c = SolidSource::from_unpremultiplied_argb(255, 30,120,255);
                                        dt.fill_rect(cbx+3.0, cby+cb_size*0.4, cb_size*0.3, cb_size*0.15, &Source::Solid(c), &DrawOptions::new());
                                        dt.fill_rect(cbx+cb_size*0.25, cby+cb_size*0.55, cb_size*0.12, cb_size*0.3, &Source::Solid(c), &DrawOptions::new());
                                    }
                                }
                                "radio" => {
                                    let r = h.min(16.0) * 0.5;
                                    let cx2 = x + w * 0.5; let cy2 = y + h * 0.5;
                                    let bs = SolidSource::from_unpremultiplied_argb(200, 80,80,80);
                                    dt.fill_rect(cx2-r, cy2-r, r*2.0, r*2.0, &Source::Solid(SolidSource::from_unpremultiplied_argb(255,255,255,255)), &DrawOptions::new());
                                    stroke_circle(dt, cx2, cy2, r, bs);
                                    if form.is_checked(&key) {
                                        let c = SolidSource::from_unpremultiplied_argb(255, 30,120,255);
                                        dt.fill_rect(cx2-r*0.4, cy2-r*0.4, r*0.8, r*0.8, &Source::Solid(c), &DrawOptions::new());
                                    }
                                }
                                _ => {
                                    let val = form.get_value(&key).to_string();
                                    let focused = form.focused.as_ref().map_or(false, |f| f == &key);
                                    let scroll_x = form.get_scroll_x(&key);
                                    if focused {
                                        let fc = SolidSource::from_unpremultiplied_argb(255, 50, 130, 250);
                                        let r = lp(&style.border_radius).min(w * 0.5).min(h * 0.5);
                                        if r > 0.0 {
                                            let bw = 2.0;
                                            let half_bw = bw * 0.5;
                                            let bx = x + half_bw;
                                            let by = y + half_bw;
                                            let bw_inner = w - bw;
                                            let bh_inner = h - bw;
                                            let br = (r - half_bw).max(0.0);

                                            let mut pb = PathBuilder::new();
                                            pb.move_to(bx + br, by);
                                            pb.line_to(bx + bw_inner - br, by);
                                            pb.quad_to(bx + bw_inner, by, bx + bw_inner, by + br);
                                            pb.line_to(bx + bw_inner, by + bh_inner - br);
                                            pb.quad_to(bx + bw_inner, by + bh_inner, bx + bw_inner - br, by + bh_inner);
                                            pb.line_to(bx + br, by + bh_inner);
                                            pb.quad_to(bx, by + bh_inner, bx, by + bh_inner - br);
                                            pb.line_to(bx, by + br);
                                            pb.quad_to(bx, by, bx + br, by);
                                            pb.close();
                                            let path = pb.finish();

                                            let stroke_style = StrokeStyle {
                                                width: bw,
                                                cap: LineCap::Butt,
                                                join: LineJoin::Miter,
                                                miter_limit: 10.0,
                                                dash_array: Vec::new(),
                                                dash_offset: 0.0,
                                            };
                                            dt.stroke(&path, &Source::Solid(fc), &stroke_style, &DrawOptions::new());
                                        } else {
                                            dt.fill_rect(x, y, 2.0, h, &Source::Solid(fc), &DrawOptions::new());
                                            dt.fill_rect(x + w - 2.0, y, 2.0, h, &Source::Solid(fc), &DrawOptions::new());
                                            dt.fill_rect(x, y, w, 2.0, &Source::Solid(fc), &DrawOptions::new());
                                            dt.fill_rect(x, y + h - 2.0, w, 2.0, &Source::Solid(fc), &DrawOptions::new());
                                        }
                                    }
                                    
                                    let mw2 = w - lp(&style.padding_left) - lp(&style.padding_right) - 2.0;
                                    let x1 = (cx + 1.0) as i32;
                                    let y1 = (y + 2.0) as i32;
                                    let x2 = (cx + 1.0 + mw2) as i32;
                                    let y2 = (y + h - 2.0) as i32;
                                    dt.push_clip_rect(IntRect::new(IntPoint::new(x1, y1), IntPoint::new(x2, y2)));

                                    if let Some((start_idx, end_idx)) = form.get_selection(&key) {
                                        if start_idx != end_idx {
                                            let s_min = start_idx.min(end_idx);
                                            let s_max = start_idx.max(end_idx);
                                            let sel_x1 = cx + 1.0 + x_at_index(&style, &val, s_min) - scroll_x;
                                            let sel_x2 = cx + 1.0 + x_at_index(&style, &val, s_max) - scroll_x;
                                            let sel_w = (sel_x2 - sel_x1).max(0.0);
                                            let sel_col = SolidSource::from_unpremultiplied_argb(100, 50, 130, 250);
                                            dt.fill_rect(sel_x1, y + 4.0, sel_w, h - 8.0, &Source::Solid(sel_col), &DrawOptions::new());
                                        }
                                    }
                                    if !val.is_empty() {
                                        let tx = cx + 1.0; let ty = y + (h - style.font_size) * 0.5;
                                        let txt_col = SolidSource::from_unpremultiplied_argb(200, 80, 80, 80);
                                        self.render_single_line_text(dt, &style, &val, tx, ty, mw2, txt_col, scroll_x);
                                        if focused && caret_on && !form.is_selected(&key) {
                                            let pos = form.cursor(&key);
                                            let tw = x_at_index(&style, &val, pos);
                                            let cx2 = cx + 1.0 + tw - scroll_x;
                                            let caret_color = SolidSource::from_unpremultiplied_argb(255, 50, 50, 50);
                                            dt.fill_rect(cx2, y + 4.0, 1.5, h - 8.0, &Source::Solid(caret_color), &DrawOptions::new());
                                        }
                                    } else if !focused {
                                        if let Some(ph) = node.get_attribute("placeholder") {
                                            let tx = cx + 1.0; let ty = y + (h - style.font_size) * 0.5;
                                            self.render_single_line_text(dt, &style, ph, tx, ty, mw2, SolidSource::from_unpremultiplied_argb(130, 160, 160, 160), 0.0);
                                        }
                                    }
                                    if focused && caret_on && val.is_empty() && !form.is_selected(&key) {
                                        let caret_color = SolidSource::from_unpremultiplied_argb(255, 50, 50, 50);
                                        dt.fill_rect(cx + 2.0 - scroll_x, y + 4.0, 1.5, h - 8.0, &Source::Solid(caret_color), &DrawOptions::new());
                                    }

                                    dt.pop_clip();
                                }
                            }
                        }
                        "button" => {
                            for child in &node.children {
                                let cptr = dom::node_ptr(child);
                                if let Some(text) = child.text() {
                                    if !text.trim().is_empty() {
                                        if let Some(clr) = layout.get(cptr) {
                                            let tx = x + clr.location.x;
                                            let ty = y + clr.location.y;
                                            let aw = clr.size.width.max(1.0);
                                            self.render_text(dt, &style, &text, tx, ty, aw);
                                        }
                                    }
                                }
                            }
                        }
                        "select" => {
                            let val = form.get_value(&key);
                            let arrow = "\u{25BC}";
                            let aw2 = style.font_size * 0.8;
                            if !val.is_empty() {
                                let tx = x + lp(&style.padding_left) + style.border.left.width;
                                let ty = y + (h - style.font_size) * 0.5;
                                let txt_col = SolidSource::from_unpremultiplied_argb(200, 41, 41, 41);
                                let mw2 = w - lp(&style.padding_left) - lp(&style.padding_right) - style.border.left.width - style.border.right.width - aw2 - 8.0;
                                self.render_single_line_text(dt, &style, val, tx, ty, mw2.max(1.0), txt_col, 0.0);
                            }
                            let ax = x + w - lp(&style.padding_right) - style.border.right.width - aw2 - 4.0;
                            let ay = y + (h - style.font_size) * 0.5;
                            let ac = SolidSource::from_unpremultiplied_argb(180, 80, 80, 80);
                            self.render_single_line_text(dt, &style, arrow, ax, ay, aw2 + 8.0, ac, 0.0);
                        }
                        "textarea" => {
                            let cx = x + lp(&style.padding_left) + style.border.left.width;
                            let val = form.get_value(&key);
                            let cy = y + lp(&style.padding_top) + style.border.top.width;
                            let aw = w - lp(&style.padding_left) - lp(&style.padding_right) - style.border.left.width - style.border.right.width;
                            if let Some((start_idx, end_idx)) = form.get_selection(&key) {
                                if start_idx != end_idx {
                                    let s_min = start_idx.min(end_idx);
                                    let s_max = start_idx.max(end_idx);
                                    let sel_x1 = cx + 1.0 + x_at_index(&style, &val, s_min);
                                    let sel_x2 = cx + 1.0 + x_at_index(&style, &val, s_max);
                                    let sel_w = (sel_x2 - sel_x1).min(x + w - sel_x1 - 2.0).max(0.0);
                                    let sel_col = SolidSource::from_unpremultiplied_argb(100, 50, 130, 250);
                                    dt.fill_rect(sel_x1, cy + 1.0, sel_w, style.font_size + 4.0, &Source::Solid(sel_col), &DrawOptions::new());
                                }
                            }
                            if !val.is_empty() || form.focused.as_ref().map_or(false, |f| f == &key) {
                                let col = SolidSource::from_unpremultiplied_argb(200, 80, 80, 80);
                                self.render_text_simple(dt, &style, val, cx+1.0, cy+1.0, aw-2.0, col);
                            }
                            if form.focused.as_ref().map_or(false, |f| f == &key) && caret_on && !form.is_selected(&key) {
                                let pos = form.cursor(&key);
                                let tw = x_at_index(&style, val, pos);
                                let cx2 = cx + 1.0 + tw.min(aw - 2.0);
                                let caret_color = SolidSource::from_unpremultiplied_argb(255, 50, 50, 50);
                                dt.fill_rect(cx2, cy + 1.0, 1.5, style.font_size + 4.0, &Source::Solid(caret_color), &DrawOptions::new());
                            }
                        }
                        _ => {
                            let pt2 = lp(&style.padding_top);
                            let bt2 = style.border.top.width;
                            for child in &node.children {
                                let cptr = dom::node_ptr(child);
                                let is_text = matches!(child.node_type, NodeType::Text(_));
                                let cl = layout.get(cptr);
                                if is_text {
                                    if let Some(text) = child.text() {
                                        if !text.trim().is_empty() {
                                            if let Some(clr) = cl {
                                                let tx = x + clr.location.x;
                                                let ty = y + clr.location.y;
                                                let aw = clr.size.width.max(1.0);
                                                let ch = lr.size.height - pt2 - lp(&style.padding_bottom) - bt2 - style.border.bottom.width;
                                                let lh = style.font_size * 1.4;
                                                let vy = if ch > lh { ty + (ch - lh) * 0.5 } else { ty };
                                                self.render_text(dt, &style, &text, tx, vy, aw);
                                            }
                                        }
                                    }
                                } else {
                                    self.paint_node(dt, styles, layout, form, child, x, y, caret_on);
                                }
                            }
                        }
                    }
                } else {
                    for c in &node.children { self.paint_node(dt, styles, layout, form, c, px, py, caret_on); }
                }
            }
            NodeType::Text(_) => {}
        }
    }

    fn render_single_line_text(&mut self, dt: &mut DrawTarget, style: &ComputedStyle, text: &str, x: f32, y: f32, max_w: f32, color_override: SolidSource, scroll_x: f32) {
        if text.is_empty() || max_w <= 0.0 { return; }
        let font = self.load_font(&style.font_family, style.font_weight)
            .or_else(|| self.load_font("Arial", style.font_weight))
            .or_else(|| self.load_font("sans-serif", style.font_weight));
        let font = match font { Some(f) => f, None => return };
        let fs = style.font_size;
        let scale = Scale::uniform(fs);
        let vm = font.v_metrics(scale);
        let ascent = vm.ascent;

        let mut cx = x - scroll_x;
        for ch in text.chars() {
            let sg = font.glyph(ch).scaled(scale);
            let aw = sg.h_metrics().advance_width;
            if cx - x + aw > max_w {
                break;
            }
            if cx + aw >= x {
                let g = sg.positioned(rpoint(cx, y + ascent));
                if let Some(bb) = g.pixel_bounding_box() {
                    let gw = bb.width() as usize;
                    let gh = bb.height() as usize;
                    if gw > 0 && gh > 0 {
                        let mut pix = vec![0u8; gw * gh];
                        g.draw(|gx, gy, cov| {
                            let ix = gx as usize; let iy = gy as usize;
                            if ix < gw && iy < gh { pix[iy * gw + ix] = (cov * 255.0) as u8; }
                        });
                        let img_data: Vec<u32> = pix.iter().map(|&a| {
                            if a == 0 { 0 } else {
                                let aa = (a as u32 * color_override.a as u32) / 255;
                                let r = (color_override.r as u32 * aa) / 255;
                                let g2 = (color_override.g as u32 * aa) / 255;
                                let b = (color_override.b as u32 * aa) / 255;
                                (aa << 24) | (r << 16) | (g2 << 8) | b
                            }
                        }).collect();
                        let img = Image { width: gw as i32, height: gh as i32, data: &img_data };
                        dt.draw_image_at(bb.min.x as f32, bb.min.y as f32, &img, &DrawOptions::new());
                    }
                }
            }
            cx += aw;
        }
    }

    fn render_text_simple(&mut self, dt: &mut DrawTarget, style: &ComputedStyle, text: &str, x: f32, y: f32, max_w: f32, color_override: SolidSource) {
        if text.is_empty() || max_w <= 0.0 { return; }
        let font = self.load_font(&style.font_family, style.font_weight).or_else(|| self.load_font("Arial", style.font_weight)).or_else(|| self.load_font("sans-serif", style.font_weight));
        let font = match font { Some(f) => f, None => return };
        let fs = style.font_size;
        let scale = Scale::uniform(fs);
        let vm = font.v_metrics(scale);
        let ascent = vm.ascent;
        let lh = fs * 1.4;
        let mut cy = y;
        let mut line = String::new();
        let mut cw = 0.0_f32;
        for word in text.split_inclusive(|c: char| c.is_whitespace()) {
            let tw = word.trim_end();
            if tw.is_empty() { continue; }
            let ww = mw(&font, scale, tw);
            let sw = if word.ends_with(' ') || word.ends_with('\t') { mw(&font, scale, " ") } else { 0.0 };
            if cw + ww > max_w && !line.is_empty() {
                draw_text_line(dt, &font, scale, &line, x, cy + ascent, color_override);
                cy += lh; line.clear(); cw = 0.0;
            }
            if !line.is_empty() { line.push(' '); cw += sw; }
            line.push_str(tw); cw += ww;
        }
        if !line.is_empty() { draw_text_line(dt, &font, scale, &line, x, cy + ascent, color_override); }
    }

    fn render_text(&mut self, dt: &mut DrawTarget, style: &ComputedStyle, text: &str, x: f32, y: f32, max_w: f32) {
        if text.trim().is_empty() || max_w <= 0.0 { return; }
        let font = self.load_font(&style.font_family, style.font_weight).or_else(|| self.load_font("Arial", style.font_weight)).or_else(|| self.load_font("sans-serif", style.font_weight));
        let font = match font { Some(f) => f, None => return };

        let fs = style.font_size;
        let scale = Scale::uniform(fs);
        let vm = font.v_metrics(scale);
        let ascent = vm.ascent;
        let lh = fs * 1.4;
        let color = style.color;

        let mut lines: Vec<String> = Vec::new();
        let mut cl = String::new();
        let mut cw = 0.0_f32;

        for word in text.split_inclusive(|c: char| c.is_whitespace()) {
            let tw = word.trim_end();
            if tw.is_empty() { continue; }
            let ww = mw(&font, scale, tw);
            let sw = if word.ends_with(' ') || word.ends_with('\t') { mw(&font, scale, " ") } else { 0.0 };
            if cw + ww > max_w && !cl.is_empty() {
                lines.push(cl.clone());
                cl = tw.to_string(); cw = ww;
            } else {
                if !cl.is_empty() { cl.push(' '); cw += sw; }
                cl.push_str(tw); cw += ww;
            }
        }
        if !cl.is_empty() { lines.push(cl); }

        let mut cy = y + ascent;
        for (i, line) in lines.iter().enumerate() {
            if i as f32 * lh > max_w * 10.0 { break; }
            let lw = mw(&font, scale, line);
            let sx = match style.text_align {
                TextAlign::Left | TextAlign::Justify => x,
                TextAlign::Center => x + (max_w - lw) / 2.0,
                TextAlign::Right => x + max_w - lw,
            };
            let mut cx = sx;
            for ch in line.chars() {
                let sg = font.glyph(ch).scaled(scale);
                let aw = sg.h_metrics().advance_width;
                let g = sg.positioned(rpoint(cx, cy));
                if let Some(bb) = g.pixel_bounding_box() {
                    let gw = bb.width() as usize;
                    let gh = bb.height() as usize;
                    if gw > 0 && gh > 0 {
                        let mut pix = vec![0u8; gw * gh];
                        g.draw(|gx, gy, cov| {
                            let ix = gx as usize; let iy = gy as usize;
                            if ix < gw && iy < gh { pix[iy * gw + ix] = (cov * 255.0) as u8; }
                        });
                        let img_data: Vec<u32> = pix.iter().map(|&a| {
                            if a == 0 { 0 } else {
                                let aa = (a as u32 * color.a as u32) / 255;
                                let r = (color.r as u32 * aa) / 255;
                                let g = (color.g as u32 * aa) / 255;
                                let b = (color.b as u32 * aa) / 255;
                                (aa << 24) | (r << 16) | (g << 8) | b
                            }
                        }).collect();
                        let img = Image { width: gw as i32, height: gh as i32, data: &img_data };
                        dt.draw_image_at(bb.min.x as f32, bb.min.y as f32, &img, &DrawOptions::new());
                    }
                }
                cx += aw;
            }
            cy += lh;
        }
    }
}

impl Node {
    fn text(&self) -> Option<String> {
        match &self.node_type { NodeType::Text(t) => Some(t.clone()), _ => None }
    }
}

fn bg(dt: &mut DrawTarget, s: &ComputedStyle, x: f32, y: f32, w: f32, h: f32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = lp(&s.border_radius).min(w * 0.5).min(h * 0.5);

    if r > 0.0 {
        let mut pb = PathBuilder::new();
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.quad_to(x + w, y, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.quad_to(x + w, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.quad_to(x, y + h, x, y + h - r);
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);
        pb.close();
        let path = pb.finish();

        if let Some(grad) = &s.background_gradient {
            let angle_rad = grad.angle.to_radians();
            let dx = angle_rad.sin();
            let dy = -angle_rad.cos();
            let len = (w * dx.abs()) + (h * dy.abs());
            let cx = x + w / 2.0;
            let cy = y + h / 2.0;
            let start_p = Point::new(cx - dx * len / 2.0, cy - dy * len / 2.0);
            let end_p = Point::new(cx + dx * len / 2.0, cy + dy * len / 2.0);

            let mut raqote_stops = Vec::new();
            for stop in &grad.stops {
                raqote_stops.push(raqote::GradientStop {
                    position: stop.position,
                    color: raqote::Color::new(stop.color.a, stop.color.r, stop.color.g, stop.color.b),
                });
            }
            let gradient = raqote::Gradient { stops: raqote_stops };
            let src = Source::new_linear_gradient(gradient, start_p, end_p, Spread::Pad);
            dt.fill(&path, &src, &DrawOptions::new());
        } else if s.background_color.a > 0 {
            let src = SolidSource::from_unpremultiplied_argb(s.background_color.a, s.background_color.r, s.background_color.g, s.background_color.b);
            dt.fill(&path, &Source::Solid(src), &DrawOptions::new());
        }
    } else {
        if let Some(grad) = &s.background_gradient {
            let angle_rad = grad.angle.to_radians();
            let dx = angle_rad.sin();
            let dy = -angle_rad.cos();
            let len = (w * dx.abs()) + (h * dy.abs());
            let cx = x + w / 2.0;
            let cy = y + h / 2.0;
            let start_p = Point::new(cx - dx * len / 2.0, cy - dy * len / 2.0);
            let end_p = Point::new(cx + dx * len / 2.0, cy + dy * len / 2.0);

            let mut raqote_stops = Vec::new();
            for stop in &grad.stops {
                raqote_stops.push(raqote::GradientStop {
                    position: stop.position,
                    color: raqote::Color::new(stop.color.a, stop.color.r, stop.color.g, stop.color.b),
                });
            }
            let gradient = raqote::Gradient { stops: raqote_stops };
            let src = Source::new_linear_gradient(gradient, start_p, end_p, Spread::Pad);
            dt.fill_rect(x, y, w, h, &src, &DrawOptions::new());
        } else if s.background_color.a > 0 {
            let src = SolidSource::from_unpremultiplied_argb(s.background_color.a, s.background_color.r, s.background_color.g, s.background_color.b);
            dt.fill_rect(x, y, w, h, &Source::Solid(src), &DrawOptions::new());
        }
    }
}

fn border(dt: &mut DrawTarget, s: &ComputedStyle, x: f32, y: f32, w: f32, h: f32) {
    let opts = DrawOptions::new();
    let b = &s.border;
    let r = lp(&s.border_radius).min(w * 0.5).min(h * 0.5);

    if r > 0.0 {
        let bw = b.top.width;
        if bw <= 0.0 { return; }

        let half_bw = bw * 0.5;
        let bx = x + half_bw;
        let by = y + half_bw;
        let bw_inner = w - bw;
        let bh_inner = h - bw;
        let br = (r - half_bw).max(0.0);

        let mut pb = PathBuilder::new();
        pb.move_to(bx + br, by);
        pb.line_to(bx + bw_inner - br, by);
        pb.quad_to(bx + bw_inner, by, bx + bw_inner, by + br);
        pb.line_to(bx + bw_inner, by + bh_inner - br);
        pb.quad_to(bx + bw_inner, by + bh_inner, bx + bw_inner - br, by + bh_inner);
        pb.line_to(bx + br, by + bh_inner);
        pb.quad_to(bx, by + bh_inner, bx, by + bh_inner - br);
        pb.line_to(bx, by + br);
        pb.quad_to(bx, by, bx + br, by);
        pb.close();
        let path = pb.finish();

        let sc = SolidSource::from_unpremultiplied_argb(b.top.color.a, b.top.color.r, b.top.color.g, b.top.color.b);
        let stroke_style = StrokeStyle {
            width: bw,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 10.0,
            dash_array: Vec::new(),
            dash_offset: 0.0,
        };
        dt.stroke(&path, &Source::Solid(sc), &stroke_style, &opts);
    } else {
        stroke_edge(dt, &b.top, x, y, w, b.top.width, true, &opts);
        stroke_edge(dt, &b.bottom, x, y + h - b.bottom.width, w, b.bottom.width, true, &opts);
        stroke_edge(dt, &b.left, x, y, b.left.width, h, false, &opts);
        stroke_edge(dt, &b.right, x + w - b.right.width, y, b.right.width, h, false, &opts);
    }
}

fn stroke_edge(dt: &mut DrawTarget, side: &BorderSide, x: f32, y: f32, w: f32, h: f32, horizontal: bool, opts: &DrawOptions) {
    if side.width <= 0.0 { return; }
    let sc = SolidSource::from_unpremultiplied_argb(side.color.a, side.color.r, side.color.g, side.color.b);
    match side.style {
        BorderLineStyle::None => {}
        BorderLineStyle::Solid => { dt.fill_rect(x, y, w, h, &Source::Solid(sc), opts); }
        BorderLineStyle::Dashed => {
            let (seg_len, gap_len): (f32, f32) = if horizontal { (8.0, 6.0) } else { (6.0, 5.0) };
            let total = if horizontal { w } else { h };
            let mut offset = 0.0;
            while offset < total {
                let seg = seg_len.min(total - offset);
                if horizontal { dt.fill_rect(x + offset, y, seg, h, &Source::Solid(sc), opts); }
                else { dt.fill_rect(x, y + offset, w, seg, &Source::Solid(sc), opts); }
                offset += seg + gap_len;
            }
        }
        BorderLineStyle::Dotted => {
            let r = (side.width / 2.0).max(1.0);
            let spacing = side.width * 1.5;
            let total = if horizontal { w } else { h };
            let mut offset = r;
            while offset < total - r {
                if horizontal { dt.fill_rect(x + offset - r, y, r * 2.0, h, &Source::Solid(sc), opts); }
                else { dt.fill_rect(x, y + offset - r, w, r * 2.0, &Source::Solid(sc), opts); }
                offset += spacing;
            }
        }
        BorderLineStyle::Double => {
            let inner_w = side.width * 0.3;
            let outer_w = side.width * 0.3;
            if horizontal {
                dt.fill_rect(x, y, w, outer_w, &Source::Solid(sc), opts);
                dt.fill_rect(x, y + side.width - inner_w, w, inner_w, &Source::Solid(sc), opts);
            } else {
                dt.fill_rect(x, y, outer_w, h, &Source::Solid(sc), opts);
                dt.fill_rect(x + side.width - inner_w, y, inner_w, h, &Source::Solid(sc), opts);
            }
        }
    }
}

fn lp(l: &crate::style::Length) -> f32 {
    match l { crate::style::Length::Px(v) => *v, _ => 0.0 }
}



fn mw(font: &Font, scale: Scale, text: &str) -> f32 {
    text.chars().map(|c| font.glyph(c).scaled(scale).h_metrics().advance_width).sum()
}

pub fn index_at_x(style: &ComputedStyle, text: &str, target_x: f32) -> usize {
    let fam = if style.font_family.is_empty() { "Arial" } else { &style.font_family };
    let data = load_font_data(fam, style.font_weight)
        .or_else(|| load_font_data("Arial", style.font_weight))
        .or_else(|| load_font_data("sans-serif", style.font_weight));
    let font = match data.and_then(|d| Font::try_from_vec(d)) {
        Some(f) => f,
        None => return (target_x / (style.font_size * 0.6)) as usize,
    };

    let scale = Scale::uniform(style.font_size);
    let mut current_x = 0.0;
    
    if target_x <= 0.0 {
        return 0;
    }

    for (i, ch) in text.chars().enumerate() {
        let aw = font.glyph(ch).scaled(scale).h_metrics().advance_width;
        if target_x < current_x + aw * 0.5 {
            return i;
        }
        current_x += aw;
        if target_x < current_x {
            return i + 1;
        }
    }
    text.chars().count()
}

pub fn x_at_index(style: &ComputedStyle, text: &str, index: usize) -> f32 {
    let fam = if style.font_family.is_empty() { "Arial" } else { &style.font_family };
    let data = load_font_data(fam, style.font_weight)
        .or_else(|| load_font_data("Arial", style.font_weight))
        .or_else(|| load_font_data("sans-serif", style.font_weight));
    let font = match data.and_then(|d| Font::try_from_vec(d)) {
        Some(f) => f,
        None => return index as f32 * style.font_size * 0.6,
    };

    let scale = Scale::uniform(style.font_size);
    let mut current_x = 0.0;
    for (i, ch) in text.chars().enumerate() {
        if i == index {
            return current_x;
        }
        let aw = font.glyph(ch).scaled(scale).h_metrics().advance_width;
        current_x += aw;
    }
    current_x
}

fn load_font_data(family: &str, weight: u16) -> Option<Vec<u8>> {
    let bold_path = if weight >= 700 {
        match family.to_lowercase().as_str() {
            "arial"|"sans-serif"|"helvetica" => Some(r"C:\Windows\Fonts\arialbd.ttf"),
            _ => None,
        }
    } else { None };
    let mut paths: Vec<&str> = Vec::new();
    if let Some(p) = bold_path { paths.push(p); }
    match family.to_lowercase().as_str() {
        "arial"|"sans-serif"|"helvetica" => {
            paths.extend_from_slice(&[r"C:\Windows\Fonts\arial.ttf", r"C:\Windows\Fonts\Arial.ttf", r"/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]);
        }
        "monospace"|"courier"|"courier new"|"consolas" => {
            paths.extend_from_slice(&[r"C:\Windows\Fonts\consola.ttf", r"C:\Windows\Fonts\cour.ttf"]);
        }
        "times"|"times new roman"|"serif" => { paths.push(r"C:\Windows\Fonts\times.ttf"); }
        "segoe ui"|"system-ui" => { paths.push(r"C:\Windows\Fonts\segoeui.ttf"); }
        _ => { paths.extend_from_slice(&[r"C:\Windows\Fonts\arial.ttf", r"C:\Windows\Fonts\segoeui.ttf"]); }
    }
    try_paths(&paths)
}

fn try_paths(paths: &[&str]) -> Option<Vec<u8>> {
    for p in paths { if let Ok(d) = std::fs::read(p) { if d.len() > 100 { return Some(d); } } }
    None
}

fn draw_text_line(dt: &mut DrawTarget, font: &Font, scale: Scale, text: &str, x: f32, y: f32, color: SolidSource) {
    let mut cx = x;
    for ch in text.chars() {
        let sg = font.glyph(ch).scaled(scale);
        let aw = sg.h_metrics().advance_width;
        let g = sg.positioned(rpoint(cx, y));
        if let Some(bb) = g.pixel_bounding_box() {
            let gw = bb.width() as usize;
            let gh = bb.height() as usize;
            if gw > 0 && gh > 0 {
                let mut pix = vec![0u8; gw * gh];
                g.draw(|gx, gy, cov| {
                    let ix = gx as usize; let iy = gy as usize;
                    if ix < gw && iy < gh { pix[iy * gw + ix] = (cov * 255.0) as u8; }
                });
                let img_data: Vec<u32> = pix.iter().map(|&a| {
                    if a == 0 { 0 } else {
                        let aa = (a as u32 * color.a as u32) / 255;
                        let r = (color.r as u32 * aa) / 255;
                        let g2 = (color.g as u32 * aa) / 255;
                        let b = (color.b as u32 * aa) / 255;
                        (aa << 24) | (r << 16) | (g2 << 8) | b
                    }
                }).collect();
                let img = Image { width: gw as i32, height: gh as i32, data: &img_data };
                dt.draw_image_at(bb.min.x as f32, bb.min.y as f32, &img, &DrawOptions::new());
            }
        }
        cx += aw;
    }
}

fn stroke_circle(dt: &mut DrawTarget, cx: f32, cy: f32, r: f32, color: SolidSource) {
    let steps = 24;
    for i in 0..steps {
        let a1 = i as f32 * std::f32::consts::TAU / steps as f32;
        let a2 = (i + 1) as f32 * std::f32::consts::TAU / steps as f32;
        let x1 = cx + a1.cos() * r; let y1 = cy + a1.sin() * r;
        let x2 = cx + a2.cos() * r; let y2 = cy + a2.sin() * r;
        let dx = x2 - x1; let dy = y2 - y1;
        let len = dx.hypot(dy);
        if len > 0.0 {
            dt.fill_rect(x1, y1, dx.max(1.0), dy.max(1.0), &Source::Solid(color), &DrawOptions::new());
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
