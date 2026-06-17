use std::collections::HashMap;
use raqote::*;
use rusttype::Font;
use crate::dom::Node;
use crate::form::FormState;
use crate::style::ComputedStyle;
use super::primitives::lp;
use super::text::{render_single_line_text, render_text_simple, render_text, x_at_index};

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
    let cx = x + lp(&style.padding_left) + style.border.left.width;
    let val = form.get_value(key).to_string();
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

    let mw2 = w - lp(&style.padding_left) - lp(&style.padding_right) - 2.0;
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

pub fn paint_textarea(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    style: &ComputedStyle,
    form: &FormState,
    key: &str,
    x: f32,
    y: f32,
    w: f32,
    _h: f32,
    caret_on: bool,
) {
    let cx = x + lp(&style.padding_left) + style.border.left.width;
    let val = form.get_value(key);
    let cy = y + lp(&style.padding_top) + style.border.top.width;
    let aw = w - lp(&style.padding_left) - lp(&style.padding_right) - style.border.left.width - style.border.right.width;
    if let Some((start_idx, end_idx)) = form.get_selection(key) {
        if start_idx != end_idx {
            let s_min = start_idx.min(end_idx);
            let s_max = start_idx.max(end_idx);
            let sel_x1 = cx + 1.0 + x_at_index(style, val, s_min);
            let sel_x2 = cx + 1.0 + x_at_index(style, val, s_max);
            let sel_w = (sel_x2 - sel_x1).min(x + w - sel_x1 - 2.0).max(0.0);
            let sel_col = SolidSource::from_unpremultiplied_argb(100, 50, 130, 250);
            dt.fill_rect(sel_x1, cy + 1.0, sel_w, style.font_size + 4.0, &Source::Solid(sel_col), &DrawOptions::new());
        }
    }
    if !val.is_empty() || form.focused.as_ref().map_or(false, |f| f == key) {
        let col = SolidSource::from_unpremultiplied_argb(200, 80, 80, 80);
        render_text_simple(dt, fonts, style, val, cx + 1.0, cy + 1.0, aw - 2.0, col);
    }
    if form.focused.as_ref().map_or(false, |f| f == key) && caret_on && !form.is_selected(key) {
        let pos = form.cursor(key);
        let tw = x_at_index(style, val, pos);
        let cx2 = cx + 1.0 + tw.min(aw - 2.0);
        let caret_color = SolidSource::from_unpremultiplied_argb(255, 50, 50, 50);
        dt.fill_rect(cx2, cy + 1.0, 1.5, style.font_size + 4.0, &Source::Solid(caret_color), &DrawOptions::new());
    }
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

