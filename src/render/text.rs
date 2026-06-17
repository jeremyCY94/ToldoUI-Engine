use std::collections::HashMap;
use raqote::{DrawTarget, SolidSource, Image, DrawOptions};
use rusttype::{Font, Scale, point as rpoint};
use crate::style::{ComputedStyle, TextAlign};

pub fn load_font(
    fonts: &mut HashMap<String, Font<'static>>,
    family: &str,
    weight: u16,
) -> Option<Font<'static>> {
    let key = format!("{}-{}", family.to_lowercase(), weight);
    if let Some(f) = fonts.get(&key) {
        return Some(f.clone());
    }
    let data = load_font_data(family, weight)?;
    let font = Font::try_from_vec(data)?;
    fonts.insert(key, font.clone());
    Some(font)
}

pub fn mw(font: &Font, scale: Scale, text: &str) -> f32 {
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

pub fn load_font_data(family: &str, weight: u16) -> Option<Vec<u8>> {
    let bold_path = if weight >= 700 {
        match family.to_lowercase().as_str() {
            "arial" | "sans-serif" | "helvetica" => Some(r"C:\Windows\Fonts\arialbd.ttf"),
            _ => None,
        }
    } else {
        None
    };
    let mut paths: Vec<&str> = Vec::new();
    if let Some(p) = bold_path {
        paths.push(p);
    }
    match family.to_lowercase().as_str() {
        "arial" | "sans-serif" | "helvetica" => {
            paths.extend_from_slice(&[
                r"C:\Windows\Fonts\arial.ttf",
                r"C:\Windows\Fonts\Arial.ttf",
                r"/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            ]);
        }
        "monospace" | "courier" | "courier new" | "consolas" => {
            paths.extend_from_slice(&[r"C:\Windows\Fonts\consola.ttf", r"C:\Windows\Fonts\cour.ttf"]);
        }
        "times" | "times new roman" | "serif" => {
            paths.push(r"C:\Windows\Fonts\times.ttf");
        }
        "segoe ui" | "system-ui" => {
            paths.push(r"C:\Windows\Fonts\segoeui.ttf");
        }
        _ => {
            paths.extend_from_slice(&[r"C:\Windows\Fonts\arial.ttf", r"C:\Windows\Fonts\segoeui.ttf"]);
        }
    }
    try_paths(&paths)
}

fn try_paths(paths: &[&str]) -> Option<Vec<u8>> {
    for p in paths {
        if let Ok(d) = std::fs::read(p) {
            if d.len() > 100 {
                return Some(d);
            }
        }
    }
    None
}

pub fn draw_text_line(
    dt: &mut DrawTarget,
    font: &Font,
    scale: Scale,
    text: &str,
    x: f32,
    y: f32,
    color: SolidSource,
) {
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
                    let ix = gx as usize;
                    let iy = gy as usize;
                    if ix < gw && iy < gh {
                        pix[iy * gw + ix] = (cov * 255.0) as u8;
                    }
                });
                let img_data: Vec<u32> = pix
                    .iter()
                    .map(|&a| {
                        if a == 0 {
                            0
                        } else {
                            let aa = (a as u32 * color.a as u32) / 255;
                            let r = (color.r as u32 * aa) / 255;
                            let g2 = (color.g as u32 * aa) / 255;
                            let b = (color.b as u32 * aa) / 255;
                            (aa << 24) | (r << 16) | (g2 << 8) | b
                        }
                    })
                    .collect();
                let img = Image {
                    width: gw as i32,
                    height: gh as i32,
                    data: &img_data,
                };
                dt.draw_image_at(bb.min.x as f32, bb.min.y as f32, &img, &DrawOptions::new());
            }
        }
        cx += aw;
    }
}

pub fn render_single_line_text(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    style: &ComputedStyle,
    text: &str,
    x: f32,
    y: f32,
    max_w: f32,
    color_override: SolidSource,
    scroll_x: f32,
) {
    if text.is_empty() || max_w <= 0.0 {
        return;
    }
    let font = load_font(fonts, &style.font_family, style.font_weight)
        .or_else(|| load_font(fonts, "Arial", style.font_weight))
        .or_else(|| load_font(fonts, "sans-serif", style.font_weight));
    let font = match font {
        Some(f) => f,
        None => return,
    };
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
                        let ix = gx as usize;
                        let iy = gy as usize;
                        if ix < gw && iy < gh {
                            pix[iy * gw + ix] = (cov * 255.0) as u8;
                        }
                    });
                    let img_data: Vec<u32> = pix
                        .iter()
                        .map(|&a| {
                            if a == 0 {
                                0
                            } else {
                                let aa = (a as u32 * color_override.a as u32) / 255;
                                let r = (color_override.r as u32 * aa) / 255;
                                let g2 = (color_override.g as u32 * aa) / 255;
                                let b = (color_override.b as u32 * aa) / 255;
                                (aa << 24) | (r << 16) | (g2 << 8) | b
                            }
                        })
                        .collect();
                    let img = Image {
                        width: gw as i32,
                        height: gh as i32,
                        data: &img_data,
                    };
                    dt.draw_image_at(bb.min.x as f32, bb.min.y as f32, &img, &DrawOptions::new());
                }
            }
        }
        cx += aw;
    }
}

pub fn render_text_simple(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    style: &ComputedStyle,
    text: &str,
    x: f32,
    y: f32,
    max_w: f32,
    color_override: SolidSource,
) {
    if text.is_empty() || max_w <= 0.0 {
        return;
    }
    let font = load_font(fonts, &style.font_family, style.font_weight)
        .or_else(|| load_font(fonts, "Arial", style.font_weight))
        .or_else(|| load_font(fonts, "sans-serif", style.font_weight));
    let font = match font {
        Some(f) => f,
        None => return,
    };
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
        if tw.is_empty() {
            continue;
        }
        let ww = mw(&font, scale, tw);
        let sw = if word.ends_with(' ') || word.ends_with('\t') {
            mw(&font, scale, " ")
        } else {
            0.0
        };
        if cw + ww > max_w && !line.is_empty() {
            draw_text_line(dt, &font, scale, &line, x, cy + ascent, color_override);
            cy += lh;
            line.clear();
            cw = 0.0;
        }
        if !line.is_empty() {
            line.push(' ');
            cw += sw;
        }
        line.push_str(tw);
        cw += ww;
    }
    if !line.is_empty() {
        draw_text_line(dt, &font, scale, &line, x, cy + ascent, color_override);
    }
}

pub fn render_text(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    style: &ComputedStyle,
    text: &str,
    x: f32,
    y: f32,
    max_w: f32,
) {
    if text.trim().is_empty() || max_w <= 0.0 {
        return;
    }
    let font = load_font(fonts, &style.font_family, style.font_weight)
        .or_else(|| load_font(fonts, "Arial", style.font_weight))
        .or_else(|| load_font(fonts, "sans-serif", style.font_weight));
    let font = match font {
        Some(f) => f,
        None => return,
    };

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
        if tw.is_empty() {
            continue;
        }
        let ww = mw(&font, scale, tw);
        let sw = if word.ends_with(' ') || word.ends_with('\t') {
            mw(&font, scale, " ")
        } else {
            0.0
        };
        if cw + ww > max_w && !cl.is_empty() {
            lines.push(cl.clone());
            cl = tw.to_string();
            cw = ww;
        } else {
            if !cl.is_empty() {
                cl.push(' ');
                cw += sw;
            }
            cl.push_str(tw);
            cw += ww;
        }
    }
    if !cl.is_empty() {
        lines.push(cl);
    }

    let mut cy = y + ascent;
    for (i, line) in lines.iter().enumerate() {
        if i as f32 * lh > max_w * 10.0 {
            break;
        }
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
                        let ix = gx as usize;
                        let iy = gy as usize;
                        if ix < gw && iy < gh {
                            pix[iy * gw + ix] = (cov * 255.0) as u8;
                        }
                    });
                    let img_data: Vec<u32> = pix
                        .iter()
                        .map(|&a| {
                            if a == 0 {
                                0
                            } else {
                                let aa = (a as u32 * color.a as u32) / 255;
                                let r = (color.r as u32 * aa) / 255;
                                let g2 = (color.g as u32 * aa) / 255;
                                let b = (color.b as u32 * aa) / 255;
                                (aa << 24) | (r << 16) | (g2 << 8) | b
                            }
                        })
                        .collect();
                    let img = Image {
                        width: gw as i32,
                        height: gh as i32,
                        data: &img_data,
                    };
                    dt.draw_image_at(bb.min.x as f32, bb.min.y as f32, &img, &DrawOptions::new());
                }
            }
            cx += aw;
        }
        cy += lh;
    }
}
