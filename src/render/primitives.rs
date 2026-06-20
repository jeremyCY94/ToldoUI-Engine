use raqote::*;
use crate::style::{BorderSide, BorderLineStyle, ComputedStyle, Length};

pub fn lp(l: &Length) -> f32 {
    match l {
        Length::Px(v) => *v,
        _ => 0.0,
    }
}

pub fn resolve_lp(l: &Length, reference: f32) -> f32 {
    match l {
        Length::Px(v) => *v,
        Length::Percent(p) => *p * reference,
        _ => 0.0,
    }
}

pub fn bg(dt: &mut DrawTarget, s: &ComputedStyle, x: f32, y: f32, w: f32, h: f32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = resolve_lp(&s.border_radius, w.min(h)).min(w * 0.5).min(h * 0.5);

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
            let src = SolidSource::from_unpremultiplied_argb(
                s.background_color.a,
                s.background_color.r,
                s.background_color.g,
                s.background_color.b,
            );
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
            let src = SolidSource::from_unpremultiplied_argb(
                s.background_color.a,
                s.background_color.r,
                s.background_color.g,
                s.background_color.b,
            );
            dt.fill_rect(x, y, w, h, &Source::Solid(src), &DrawOptions::new());
        }
    }
}

pub fn border(dt: &mut DrawTarget, s: &ComputedStyle, x: f32, y: f32, w: f32, h: f32) {
    let opts = DrawOptions::new();
    let b = &s.border;
    let r = resolve_lp(&s.border_radius, w.min(h)).min(w * 0.5).min(h * 0.5);

    if r > 0.0 {
        let bw = b.top.width;
        if bw <= 0.0 {
            return;
        }

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

        let sc = SolidSource::from_unpremultiplied_argb(
            b.top.color.a,
            b.top.color.r,
            b.top.color.g,
            b.top.color.b,
        );
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

fn stroke_edge(
    dt: &mut DrawTarget,
    side: &BorderSide,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    horizontal: bool,
    opts: &DrawOptions,
) {
    if side.width <= 0.0 {
        return;
    }
    let sc = SolidSource::from_unpremultiplied_argb(
        side.color.a,
        side.color.r,
        side.color.g,
        side.color.b,
    );
    match side.style {
        BorderLineStyle::None => {}
        BorderLineStyle::Solid => {
            dt.fill_rect(x, y, w, h, &Source::Solid(sc), opts);
        }
        BorderLineStyle::Dashed => {
            let (seg_len, gap_len): (f32, f32) = if horizontal { (8.0, 6.0) } else { (6.0, 5.0) };
            let total = if horizontal { w } else { h };
            let mut offset = 0.0;
            while offset < total {
                let seg = seg_len.min(total - offset);
                if horizontal {
                    dt.fill_rect(x + offset, y, seg, h, &Source::Solid(sc), opts);
                } else {
                    dt.fill_rect(x, y + offset, w, seg, &Source::Solid(sc), opts);
                }
                offset += seg + gap_len;
            }
        }
        BorderLineStyle::Dotted => {
            let r = (side.width / 2.0).max(1.0);
            let spacing = side.width * 1.5;
            let total = if horizontal { w } else { h };
            let mut offset = r;
            while offset < total - r {
                if horizontal {
                    dt.fill_rect(x + offset - r, y, r * 2.0, h, &Source::Solid(sc), opts);
                } else {
                    dt.fill_rect(x, y + offset - r, w, r * 2.0, &Source::Solid(sc), opts);
                }
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

pub fn stroke_circle(dt: &mut DrawTarget, cx: f32, cy: f32, r: f32, color: SolidSource) {
    let steps = 24;
    for i in 0..steps {
        let a1 = i as f32 * std::f32::consts::TAU / steps as f32;
        let a2 = (i + 1) as f32 * std::f32::consts::TAU / steps as f32;
        let x1 = cx + a1.cos() * r;
        let y1 = cy + a1.sin() * r;
        let x2 = cx + a2.cos() * r;
        let y2 = cy + a2.sin() * r;
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = dx.hypot(dy);
        if len > 0.0 {
            dt.fill_rect(x1, y1, dx.max(1.0), dy.max(1.0), &Source::Solid(color), &DrawOptions::new());
        }
    }
}
