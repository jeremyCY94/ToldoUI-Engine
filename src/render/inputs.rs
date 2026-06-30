use std::collections::HashMap;
use raqote::*;
use rusttype::Font;
use crate::dom::Node;
use crate::form::FormState;
use crate::style::ComputedStyle;
use super::primitives::lp;
use super::text::{render_single_line_text, render_text, x_at_index};

pub fn paint_checkbox(
    dt: &mut DrawTarget,
    style: &ComputedStyle,
    form: &FormState,
    key: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let cb_size = h.min(18.0);
    let cbx = x + (w - cb_size) * 0.5;
    let cby = y + (h - cb_size) * 0.5;

    let is_checked = form.is_checked(key);
    let r = lp(&style.border_radius).min(cb_size * 0.5);

    let bg_color = if is_checked {
        if style.background_color.a > 0 {
            SolidSource::from_unpremultiplied_argb(
                style.background_color.a,
                style.background_color.r,
                style.background_color.g,
                style.background_color.b,
            )
        } else {
            SolidSource::from_unpremultiplied_argb(255, 33, 150, 243)
        }
    } else {
        if style.background_color.a > 0 {
            SolidSource::from_unpremultiplied_argb(
                style.background_color.a,
                style.background_color.r,
                style.background_color.g,
                style.background_color.b,
            )
        } else {
            SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)
        }
    };

    let border_color = if is_checked {
        if style.background_color.a > 0 {
            SolidSource::from_unpremultiplied_argb(
                style.background_color.a,
                style.background_color.r,
                style.background_color.g,
                style.background_color.b,
            )
        } else {
            SolidSource::from_unpremultiplied_argb(255, 30, 136, 229)
        }
    } else {
        if style.border.top.width > 0.0 {
            let bc = &style.border.top.color;
            SolidSource::from_unpremultiplied_argb(bc.a, bc.r, bc.g, bc.b)
        } else {
            SolidSource::from_unpremultiplied_argb(255, 200, 200, 200)
        }
    };

    let mut pb = PathBuilder::new();
    if r > 0.0 {
        pb.move_to(cbx + r, cby);
        pb.line_to(cbx + cb_size - r, cby);
        pb.quad_to(cbx + cb_size, cby, cbx + cb_size, cby + r);
        pb.line_to(cbx + cb_size, cby + cb_size - r);
        pb.quad_to(cbx + cb_size, cby + cb_size, cbx + cb_size - r, cby + cb_size);
        pb.line_to(cbx + r, cby + cb_size);
        pb.quad_to(cbx, cby + cb_size, cbx, cby + cb_size - r);
        pb.line_to(cbx, cby + r);
        pb.quad_to(cbx, cby, cbx + r, cby);
        pb.close();
    } else {
        pb.rect(cbx, cby, cb_size, cb_size);
    }
    let path = pb.finish();

    dt.fill(&path, &Source::Solid(bg_color), &DrawOptions::new());

    let stroke_style = StrokeStyle {
        width: 1.0,
        cap: LineCap::Butt,
        join: LineJoin::Miter,
        miter_limit: 10.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    dt.stroke(&path, &Source::Solid(border_color), &stroke_style, &DrawOptions::new());

    if is_checked {
        let mut pb_check = PathBuilder::new();
        pb_check.move_to(cbx + cb_size * 0.25, cby + cb_size * 0.5);
        pb_check.line_to(cbx + cb_size * 0.45, cby + cb_size * 0.7);
        pb_check.line_to(cbx + cb_size * 0.75, cby + cb_size * 0.3);
        let path_check = pb_check.finish();

        let check_stroke = StrokeStyle {
            width: 2.0,
            cap: LineCap::Round,
            join: LineJoin::Round,
            miter_limit: 10.0,
            dash_array: Vec::new(),
            dash_offset: 0.0,
        };
        let check_color = SolidSource::from_unpremultiplied_argb(255, 255, 255, 255);
        dt.stroke(&path_check, &Source::Solid(check_color), &check_stroke, &DrawOptions::new());
    }
}

pub fn paint_radio(
    dt: &mut DrawTarget,
    style: &ComputedStyle,
    form: &FormState,
    key: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    let r = (h.min(18.0) * 0.5) - 1.0;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;

    let is_checked = form.is_checked(key);

    let bg_color = if style.background_color.a > 0 {
        SolidSource::from_unpremultiplied_argb(
            style.background_color.a,
            style.background_color.r,
            style.background_color.g,
            style.background_color.b,
        )
    } else {
        SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)
    };

    let border_color = if is_checked {
        SolidSource::from_unpremultiplied_argb(255, 33, 150, 243)
    } else if style.border.top.width > 0.0 {
        let bc = &style.border.top.color;
        SolidSource::from_unpremultiplied_argb(bc.a, bc.r, bc.g, bc.b)
    } else {
        SolidSource::from_unpremultiplied_argb(255, 200, 200, 200)
    };

    let main_circle = draw_circle_path(cx, cy, r);
    dt.fill(&main_circle, &Source::Solid(bg_color), &DrawOptions::new());

    let stroke_style = StrokeStyle {
        width: 1.5,
        cap: LineCap::Butt,
        join: LineJoin::Miter,
        miter_limit: 10.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    dt.stroke(&main_circle, &Source::Solid(border_color), &stroke_style, &DrawOptions::new());

    if is_checked {
        let inner_r = r * 0.55;
        let inner_circle = draw_circle_path(cx, cy, inner_r);
        let active_color = SolidSource::from_unpremultiplied_argb(255, 33, 150, 243);
        dt.fill(&inner_circle, &Source::Solid(active_color), &DrawOptions::new());
    }
}

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


pub fn paint_input_text(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    style: &ComputedStyle,
    form: &FormState,
    node: &Node,
    key: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    caret_on: bool,
) {
    let is_date = node.tag_name() == Some("input") && node.get_attribute("type") == Some("date");
    let is_time = node.tag_name() == Some("input") && node.get_attribute("type") == Some("time");
    let icon_width = 24.0;
    let cx = x + lp(&style.padding_left) + style.border.left.width + if is_date { icon_width } else { 0.0 };
    let raw_val = form.get_value(key).to_string();
    let val = if is_date && raw_val.is_empty() {
        node.get_attribute("format").unwrap_or("dd/MM/yyyy").to_string()
    } else if is_time && raw_val.is_empty() {
        node.get_attribute("format").unwrap_or("HH:mm").to_string()
    } else {
        raw_val
    };
    let focused = form.focused.as_ref().map_or(false, |f| f == key);
    let scroll_x = form.get_scroll_x(key);

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

    let mw2 = w - lp(&style.padding_left) - lp(&style.padding_right) - 2.0 - if is_date { icon_width } else { 0.0 };

    if is_date {
        let icon_size = 14.0f32;
        let ix = x + lp(&style.padding_left) + style.border.left.width + 4.0;
        let iy = y + (h - icon_size) * 0.5;

        let icon_color = if focused {
            SolidSource::from_unpremultiplied_argb(255, 50, 130, 250)
        } else {
            SolidSource::from_unpremultiplied_argb(255, 120, 120, 120)
        };

        let mut pb = PathBuilder::new();
        // Body outline
        pb.rect(ix, iy + 2.0, icon_size, icon_size - 2.0);
        // Header line
        pb.move_to(ix, iy + 5.0);
        pb.line_to(ix + icon_size, iy + 5.0);

        // Binder rings
        pb.rect(ix + 3.0, iy, 1.5, 3.0);
        pb.rect(ix + icon_size - 4.5, iy, 1.5, 3.0);

        // Grid points
        let dot_w = 1.0;
        let dot_h = 1.0;
        pb.rect(ix + 3.0, iy + 7.0, dot_w, dot_h);
        pb.rect(ix + 6.5, iy + 7.0, dot_w, dot_h);
        pb.rect(ix + 10.0, iy + 7.0, dot_w, dot_h);
        pb.rect(ix + 3.0, iy + 10.0, dot_w, dot_h);
        pb.rect(ix + 6.5, iy + 10.0, dot_w, dot_h);
        pb.rect(ix + 10.0, iy + 10.0, dot_w, dot_h);

        let path = pb.finish();
        let stroke_style = StrokeStyle {
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 10.0,
            dash_array: Vec::new(),
            dash_offset: 0.0,
        };
        dt.stroke(&path, &Source::Solid(icon_color), &stroke_style, &DrawOptions::new());
    }

    let x1 = (cx + 1.0) as i32;
    let y1 = (y + 2.0) as i32;
    let x2 = (cx + 1.0 + mw2) as i32;
    let y2 = (y + h - 2.0) as i32;
    dt.push_clip_rect(IntRect::new(IntPoint::new(x1, y1), IntPoint::new(x2, y2)));

    if let Some((start_idx, end_idx)) = form.get_selection(key) {
        if start_idx != end_idx {
            let s_min = start_idx.min(end_idx);
            let s_max = start_idx.max(end_idx);
            let sel_x1 = cx + 1.0 + x_at_index(style, &val, s_min) - scroll_x;
            let sel_x2 = cx + 1.0 + x_at_index(style, &val, s_max) - scroll_x;
            let sel_w = (sel_x2 - sel_x1).max(0.0);
            let sel_col = SolidSource::from_unpremultiplied_argb(100, 50, 130, 250);
            dt.fill_rect(sel_x1, y + 4.0, sel_w, h - 8.0, &Source::Solid(sel_col), &DrawOptions::new());
        }
    }
    if !val.is_empty() {
        let tx = cx + 1.0;
        let ty = y + (h - style.font_size) * 0.5;
        let txt_col = SolidSource::from_unpremultiplied_argb(200, 80, 80, 80);
        render_single_line_text(dt, fonts, style, &val, tx, ty, mw2, txt_col, scroll_x);
        if focused && caret_on && !form.is_selected(key) {
            let pos = form.cursor(key);
            let tw = x_at_index(style, &val, pos);
            let cx2 = cx + 1.0 + tw - scroll_x;
            let caret_color = SolidSource::from_unpremultiplied_argb(255, 50, 50, 50);
            dt.fill_rect(cx2, y + 4.0, 1.5, h - 8.0, &Source::Solid(caret_color), &DrawOptions::new());
        }
    } else if !focused {
        if let Some(ph) = node.get_attribute("placeholder") {
            let tx = cx + 1.0;
            let ty = y + (h - style.font_size) * 0.5;
            render_single_line_text(
                dt,
                fonts,
                style,
                ph,
                tx,
                ty,
                mw2,
                SolidSource::from_unpremultiplied_argb(130, 160, 160, 160),
                0.0,
            );
        }
    }
    if focused && caret_on && val.is_empty() && !form.is_selected(key) {
        let caret_color = SolidSource::from_unpremultiplied_argb(255, 50, 50, 50);
        dt.fill_rect(cx + 2.0 - scroll_x, y + 4.0, 1.5, h - 8.0, &Source::Solid(caret_color), &DrawOptions::new());
    }

    dt.pop_clip();
}

#[derive(Debug)]
struct LayoutLine {
    start_idx: usize,
    text: String,
}

fn wrap_textarea_text(
    text: &str,
    font: &Font<'static>,
    scale: rusttype::Scale,
    max_w: f32,
) -> Vec<LayoutLine> {
    if max_w <= 0.0 {
        return vec![LayoutLine {
            start_idx: 0,
            text: String::new(),
        }];
    }

    let mut lines = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    
    let mut paragraphs = Vec::new();
    let mut current_paragraph = Vec::new();
    let mut p_start = 0;
    
    for (i, &ch) in chars.iter().enumerate() {
        if ch == '\n' {
            paragraphs.push((p_start, current_paragraph.clone()));
            current_paragraph.clear();
            p_start = i + 1;
        } else {
            current_paragraph.push(ch);
        }
    }
    paragraphs.push((p_start, current_paragraph));

    for (p_start_idx, p_chars) in paragraphs {
        if p_chars.is_empty() {
            lines.push(LayoutLine {
                start_idx: p_start_idx,
                text: String::new(),
            });
            continue;
        }

        let mut words = Vec::new();
        let mut word_buf = String::new();
        let mut word_start = 0;
        
        for (idx, &ch) in p_chars.iter().enumerate() {
            word_buf.push(ch);
            if ch.is_whitespace() {
                words.push((word_start, word_buf.clone()));
                word_buf.clear();
                word_start = idx + 1;
            }
        }
        if !word_buf.is_empty() {
            words.push((word_start, word_buf));
        }

        let mut current_line_text = String::new();
        let mut current_line_start = p_start_idx;
        let mut current_line_w = 0.0;

        for (w_offset, word) in words {
            let word_w: f32 = word.chars().map(|c| font.glyph(c).scaled(scale).h_metrics().advance_width).sum();
            
            if word_w > max_w {
                if !current_line_text.is_empty() {
                    lines.push(LayoutLine {
                        start_idx: current_line_start,
                        text: current_line_text.clone(),
                    });
                    current_line_text.clear();
                    current_line_w = 0.0;
                }
                
                current_line_start = p_start_idx + w_offset;
                for (ch_idx, ch) in word.chars().enumerate() {
                    let ch_w = font.glyph(ch).scaled(scale).h_metrics().advance_width;
                    if current_line_w + ch_w > max_w && !current_line_text.is_empty() {
                        lines.push(LayoutLine {
                            start_idx: current_line_start,
                            text: current_line_text.clone(),
                        });
                        current_line_text.clear();
                        current_line_start = p_start_idx + w_offset + ch_idx;
                        current_line_w = 0.0;
                    }
                    current_line_text.push(ch);
                    current_line_w += ch_w;
                }
            } else {
                if current_line_w + word_w > max_w && !current_line_text.is_empty() {
                    lines.push(LayoutLine {
                        start_idx: current_line_start,
                        text: current_line_text.clone(),
                    });
                    current_line_text = word;
                    current_line_start = p_start_idx + w_offset;
                    current_line_w = word_w;
                } else {
                    current_line_text.push_str(&word);
                    current_line_w += word_w;
                }
            }
        }
        
        if !current_line_text.is_empty() {
            lines.push(LayoutLine {
                start_idx: current_line_start,
                text: current_line_text,
            });
        }
    }
    
    lines
}

fn get_caret_position(
    lines: &[LayoutLine],
    pos: usize,
    font: &Font<'static>,
    scale: rusttype::Scale,
) -> (usize, f32) {
    if lines.is_empty() {
        return (0, 0.0);
    }
    
    let mut best_line_idx = 0;
    for (i, line) in lines.iter().enumerate() {
        if pos >= line.start_idx {
            best_line_idx = i;
        }
    }
    
    let line = &lines[best_line_idx];
    let offset_chars = pos.saturating_sub(line.start_idx);
    let mut x_offset = 0.0;
    for (i, ch) in line.text.chars().enumerate() {
        if i >= offset_chars {
            break;
        }
        x_offset += font.glyph(ch).scaled(scale).h_metrics().advance_width;
    }
    
    (best_line_idx, x_offset)
}

pub fn textarea_index_at_point(
    style: &ComputedStyle,
    text: &str,
    target_x: f32,
    target_y: f32,
    max_w: f32,
) -> usize {
    let fam = if style.font_family.is_empty() { "Arial" } else { &style.font_family };
    let data = super::text::load_font_data(fam, style.font_weight)
        .or_else(|| super::text::load_font_data("Arial", style.font_weight))
        .or_else(|| super::text::load_font_data("sans-serif", style.font_weight));
    let font = match data.and_then(|d| Font::try_from_vec(d)) {
        Some(f) => f,
        None => return (target_x / (style.font_size * 0.6)) as usize,
    };
    
    let scale = rusttype::Scale::uniform(style.font_size);
    let lh = style.font_size * 1.4;
    
    let lines = wrap_textarea_text(text, &font, scale, max_w);
    if lines.is_empty() {
        return 0;
    }
    
    let clicked_line_idx = (target_y / lh).max(0.0) as usize;
    let line_idx = clicked_line_idx.min(lines.len() - 1);
    let line = &lines[line_idx];
    
    let mut current_x = 0.0;
    for (i, ch) in line.text.chars().enumerate() {
        let aw = font.glyph(ch).scaled(scale).h_metrics().advance_width;
        if target_x < current_x + aw * 0.5 {
            return line.start_idx + i;
        }
        current_x += aw;
        if target_x < current_x {
            return line.start_idx + i + 1;
        }
    }
    line.start_idx + line.text.chars().count()
}

pub fn paint_textarea(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    style: &ComputedStyle,
    form: &FormState,
    key: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    caret_on: bool,
) {
    let cx = x + lp(&style.padding_left) + style.border.left.width;
    let cy = y + lp(&style.padding_top) + style.border.top.width;
    let aw = w - lp(&style.padding_left) - lp(&style.padding_right) - style.border.left.width - style.border.right.width;
    let ah = h - lp(&style.padding_top) - lp(&style.padding_bottom) - style.border.top.width - style.border.bottom.width;
    
    if aw <= 0.0 || ah <= 0.0 {
        return;
    }

    let focused = form.focused.as_ref().map_or(false, |f| f == key);
    
    // Draw the focus border
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

    let val = form.get_value(key);
    
    // Load font
    let font = super::text::load_font(fonts, &style.font_family, style.font_weight)
        .or_else(|| super::text::load_font(fonts, "Arial", style.font_weight))
        .or_else(|| super::text::load_font(fonts, "sans-serif", style.font_weight));
    let font = match font {
        Some(f) => f,
        None => return,
    };
    
    let scale = rusttype::Scale::uniform(style.font_size);
    let vm = font.v_metrics(scale);
    let ascent = vm.ascent;
    let lh = style.font_size * 1.4;

    // Wrap text into layout lines
    let lines = wrap_textarea_text(val, &font, scale, aw - 2.0);

    // Push clip rect to keep rendering inside content bounds
    dt.push_clip_rect(IntRect::new(
        IntPoint::new((cx) as i32, (cy) as i32),
        IntPoint::new((cx + aw) as i32, (cy + ah) as i32),
    ));

    // Render selection highlight
    if let Some((start_idx, end_idx)) = form.get_selection(key) {
        if start_idx != end_idx {
            let s_min = start_idx.min(end_idx);
            let s_max = start_idx.max(end_idx);
            
            for (line_idx, line) in lines.iter().enumerate() {
                let line_len = line.text.chars().count();
                let line_end = line.start_idx + line_len;
                let line_y = cy + 1.0 + line_idx as f32 * lh;
                
                let sel_line_start = s_min.max(line.start_idx);
                let sel_line_end = s_max.min(line_end + 1);
                
                if sel_line_start < sel_line_end {
                    let offset_start = sel_line_start - line.start_idx;
                    let offset_end = (sel_line_end - line.start_idx).min(line_len);
                    
                    let mut sel_x1 = 0.0;
                    for ch in line.text.chars().take(offset_start) {
                        sel_x1 += font.glyph(ch).scaled(scale).h_metrics().advance_width;
                    }
                    
                    let mut sel_x2 = sel_x1;
                    for ch in line.text.chars().skip(offset_start).take(offset_end - offset_start) {
                        sel_x2 += font.glyph(ch).scaled(scale).h_metrics().advance_width;
                    }
                    
                    if sel_line_end > line_end {
                        sel_x2 += 8.0;
                    }
                    
                    let x_draw = cx + 1.0 + sel_x1;
                    let w_draw = (sel_x2 - sel_x1).min(cx + aw - x_draw).max(0.0);
                    let sel_col = SolidSource::from_unpremultiplied_argb(100, 50, 130, 250);
                    dt.fill_rect(x_draw, line_y, w_draw, lh, &Source::Solid(sel_col), &DrawOptions::new());
                }
            }
        }
    }

    // Render text lines
    let text_color = SolidSource::from_unpremultiplied_argb(200, 80, 80, 80);
    for (line_idx, line) in lines.iter().enumerate() {
        let line_y = cy + 1.0 + line_idx as f32 * lh;
        if !line.text.is_empty() {
            super::text::draw_text_line(dt, &font, scale, &line.text, cx + 1.0, line_y + ascent, text_color);
        }
    }

    // Render caret
    if focused && caret_on && !form.is_selected(key) {
        let pos = form.cursor(key);
        let (caret_line_idx, caret_x_offset) = get_caret_position(&lines, pos, &font, scale);
        
        let caret_y = cy + 1.0 + caret_line_idx as f32 * lh;
        let cx2 = cx + 1.0 + caret_x_offset.min(aw - 2.0);
        let caret_color = SolidSource::from_unpremultiplied_argb(255, 50, 50, 50);
        dt.fill_rect(cx2, caret_y, 1.5, lh, &Source::Solid(caret_color), &DrawOptions::new());
    }

    dt.pop_clip();
}

pub fn paint_button(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    style: &ComputedStyle,
    node: &Node,
    layout: &crate::layout::LayoutEngine,
    x: f32,
    y: f32,
) {
    for child in &node.children {
        let cptr = crate::dom::node_ptr(child);
        if let Some(text) = child.text() {
            if !text.trim().is_empty() {
                if let Some(clr) = layout.get(cptr) {
                    let tx = x + clr.location.x;
                    let ty = y + clr.location.y;
                    let aw = clr.size.width.max(1.0);
                    render_text(dt, fonts, style, &text, tx, ty, aw);
                }
            }
        }
    }
}

