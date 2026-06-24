use std::collections::HashMap;
use std::rc::Rc;

use raqote::*;
use rusttype::Font;

use crate::dom::{self, Node, NodeType};
use crate::form::FormState;
use crate::layout::LayoutEngine;
use crate::style::StyleMap;

use super::primitives;
use super::text;
use super::inputs;
use super::select;
use super::overlay;
use super::date_dropdown;

// Re-export methods used by input.rs and app.rs to keep compatibility
pub use super::text::{index_at_x, x_at_index};
pub use super::inputs::textarea_index_at_point;

pub struct LoadedImage {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u32>,
}

pub struct Painter {
    fonts: HashMap<String, Font<'static>>,
    image_cache: HashMap<String, LoadedImage>,
}

impl Painter {
    pub fn new() -> Self {
        Painter {
            fonts: HashMap::new(),
            image_cache: HashMap::new(),
        }
    }

    pub fn paint(
        &mut self,
        dt: &mut DrawTarget,
        styles: &StyleMap,
        layout: &LayoutEngine,
        form: &FormState,
        root: Rc<Node>,
        scroll_y: f32,
        content_h: f32,
        vw: f32,
        vh: f32,
        caret_on: bool,
        mouse_x: f32,
        mouse_y: f32,
        dragging_scrollbar: bool,
        loading: bool,
        modal: &Option<overlay::ModalState>,
    ) {
        dt.clear(SolidSource::from_unpremultiplied_argb(255, 255, 255, 255));
        self.paint_node(dt, styles, layout, form, &root, 0.0, -scroll_y, caret_on);
        if content_h > vh {
            let sb_w = 8.0;
            let sb_x = vw - sb_w - 4.0;
            let track_h = vh - 8.0;
            let thumb_h = (vh / content_h * track_h).max(30.0);
            let thumb_y = 4.0 + (scroll_y / (content_h - vh)) * (track_h - thumb_h);

            let is_hovered = mouse_x >= sb_x - 4.0 && mouse_x <= vw && mouse_y >= 0.0 && mouse_y <= vh;

            let mut pb_track = PathBuilder::new();
            let r = sb_w * 0.5;
            pb_track.move_to(sb_x + r, 4.0);
            pb_track.line_to(sb_x + sb_w - r, 4.0);
            pb_track.quad_to(sb_x + sb_w, 4.0, sb_x + sb_w, 4.0 + r);
            pb_track.line_to(sb_x + sb_w, 4.0 + track_h - r);
            pb_track.quad_to(sb_x + sb_w, 4.0 + track_h, sb_x + sb_w - r, 4.0 + track_h);
            pb_track.line_to(sb_x + r, 4.0 + track_h);
            pb_track.quad_to(sb_x, 4.0 + track_h, sb_x, 4.0 + track_h - r);
            pb_track.line_to(sb_x, 4.0 + r);
            pb_track.quad_to(sb_x, 4.0, sb_x + r, 4.0);
            pb_track.close();
            let path_track = pb_track.finish();

            let track_alpha = if is_hovered || dragging_scrollbar { 25 } else { 8 };
            let track_color = SolidSource::from_unpremultiplied_argb(track_alpha, 0, 0, 0);
            dt.fill(&path_track, &Source::Solid(track_color), &DrawOptions::new());

            let mut pb_thumb = PathBuilder::new();
            pb_thumb.move_to(sb_x + r, thumb_y);
            pb_thumb.line_to(sb_x + sb_w - r, thumb_y);
            pb_thumb.quad_to(sb_x + sb_w, thumb_y, sb_x + sb_w, thumb_y + r);
            pb_thumb.line_to(sb_x + sb_w, thumb_y + thumb_h - r);
            pb_thumb.quad_to(sb_x + sb_w, thumb_y + thumb_h, sb_x + sb_w - r, thumb_y + thumb_h);
            pb_thumb.line_to(sb_x + r, thumb_y + thumb_h);
            pb_thumb.quad_to(sb_x, thumb_y + thumb_h, sb_x, thumb_y + thumb_h - r);
            pb_thumb.line_to(sb_x, thumb_y + r);
            pb_thumb.quad_to(sb_x, thumb_y, sb_x + r, thumb_y);
            pb_thumb.close();
            let path_thumb = pb_thumb.finish();

            let thumb_alpha = if dragging_scrollbar {
                135
            } else if is_hovered {
                95
            } else {
                55
            };
            let thumb_color = SolidSource::from_unpremultiplied_argb(thumb_alpha, 0, 0, 0);
            dt.fill(&path_thumb, &Source::Solid(thumb_color), &DrawOptions::new());
        }

        // Draw the select dropdown overlay if a select is focused
        select::paint_select_dropdown_overlay(
            dt,
            &mut self.fonts,
            styles,
            layout,
            form,
            &root,
            scroll_y,
            mouse_x,
            mouse_y,
        );

        // Draw the date dropdown overlay if a date input is focused
        date_dropdown::paint_date_dropdown_overlay(
            dt,
            &mut self.fonts,
            styles,
            layout,
            form,
            &root,
            scroll_y,
            mouse_x,
            mouse_y,
        );

        if loading {
            overlay::paint_loading_overlay(dt, vw, vh);
        }

        if let Some(m) = modal {
            overlay::paint_modal_overlay(dt, &mut self.fonts, m, vw, vh, mouse_x, mouse_y);
        }
    }

    fn paint_node(
        &mut self,
        dt: &mut DrawTarget,
        styles: &StyleMap,
        layout: &LayoutEngine,
        form: &FormState,
        node: &Rc<Node>,
        px: f32,
        py: f32,
        caret_on: bool,
    ) {
        let ptr = dom::node_ptr(node);
        match &node.node_type {
            NodeType::Document => {
                for c in &node.children {
                    self.paint_node(dt, styles, layout, form, c, px, py, caret_on);
                }
            }
            NodeType::Element(_) => {
                let style = match styles.get(&ptr) {
                    Some(s) => s.clone(),
                    None => {
                        for c in &node.children {
                            self.paint_node(dt, styles, layout, form, c, px, py, caret_on);
                        }
                        return;
                    }
                };
                if style.display == taffy::Display::None || !style.visibility {
                    return;
                }

                if let Some(lr) = layout.get(ptr) {
                    let x = px + lr.location.x;
                    let y = py + lr.location.y;
                    let w = lr.size.width;
                    let h = lr.size.height;

                    let tag = node.tag_name().unwrap_or("");
                    let itype = if tag == "input" {
                        node.get_attribute("type").unwrap_or("text")
                    } else {
                        ""
                    };

                    let is_checkbox_or_radio = tag == "input" && (itype == "checkbox" || itype == "radio");

                    if !is_checkbox_or_radio {
                        primitives::bg(dt, &style, x, y, w, h);
                        primitives::border(dt, &style, x, y, w, h);
                    }

                    let key = format!("{:p}", ptr);

                    match tag {
                        "input" => {
                            let itype = node.get_attribute("type").unwrap_or("text");
                            match itype {
                                "checkbox" => {
                                    inputs::paint_checkbox(dt, &style, form, &key, x, y, w, h);
                                }
                                "radio" => {
                                    inputs::paint_radio(dt, &style, form, &key, x, y, w, h);
                                }
                                _ => {
                                    inputs::paint_input_text(
                                        dt,
                                        &mut self.fonts,
                                        &style,
                                        form,
                                        node,
                                        &key,
                                        x,
                                        y,
                                        w,
                                        h,
                                        caret_on,
                                    );
                                }
                            }
                        }
                        "button" => {
                            inputs::paint_button(dt, &mut self.fonts, &style, node, layout, x, y);
                        }
                        "select" => {
                            select::paint_select(dt, &mut self.fonts, &style, form, &key, x, y, w, h);
                        }
                        "textarea" => {
                            inputs::paint_textarea(
                                dt,
                                &mut self.fonts,
                                &style,
                                form,
                                &key,
                                x,
                                y,
                                w,
                                h,
                                caret_on,
                            );
                        }
                        "img" => {
                            if let Some(src) = node.get_attribute("src") {
                                let target_w = w.max(1.0) as u32;
                                let target_h = h.max(1.0) as u32;
                                if let Some(img) = self.get_or_load_image(src, target_w, target_h) {
                                    let image = raqote::Image {
                                        width: img.width,
                                        height: img.height,
                                        data: &img.pixels,
                                    };
                                    let r = primitives::resolve_lp(&style.border_radius, w.min(h)).min(w * 0.5).min(h * 0.5);
                                    let has_clip = r > 0.0;
                                    if has_clip {
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
                                        dt.push_clip(&pb.finish());
                                    }
                                    dt.draw_image_with_size_at(w, h, x, y, &image, &DrawOptions::new());
                                    if has_clip {
                                        dt.pop_clip();
                                    }
                                } else {
                                    let mut pb = PathBuilder::new();
                                    let r = primitives::resolve_lp(&style.border_radius, w.min(h)).min(w * 0.5).min(h * 0.5);
                                    if r > 0.0 {
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
                                        dt.fill(&path, &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 240, 240, 240)), &DrawOptions::new());
                                        let stroke_style = StrokeStyle {
                                            width: 1.0,
                                            cap: LineCap::Butt,
                                            join: LineJoin::Miter,
                                            miter_limit: 10.0,
                                            dash_array: vec![4.0, 4.0],
                                            dash_offset: 0.0,
                                        };
                                        dt.stroke(&path, &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 180, 180, 180)), &stroke_style, &DrawOptions::new());
                                    } else {
                                        dt.fill_rect(x, y, w, h, &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 240, 240, 240)), &DrawOptions::new());
                                        pb.rect(x, y, w, h);
                                        let path = pb.finish();
                                        let stroke_style = StrokeStyle {
                                            width: 1.0,
                                            cap: LineCap::Butt,
                                            join: LineJoin::Miter,
                                            miter_limit: 10.0,
                                            dash_array: vec![4.0, 4.0],
                                            dash_offset: 0.0,
                                        };
                                        dt.stroke(&path, &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 180, 180, 180)), &stroke_style, &DrawOptions::new());
                                    }
                                    let mut font_style = style.clone();
                                    font_style.font_size = 11.0;
                                    font_style.color = crate::style::Color::new(120, 120, 120, 255);
                                    let err_text = format!("Error: {}", src);
                                    let tw = text::x_at_index(&font_style, &err_text, err_text.chars().count());
                                    let tx = x + (w - tw) * 0.5;
                                    let ty = y + (h - font_style.font_size) * 0.5;
                                    text::render_text(dt, &mut self.fonts, &font_style, &err_text, tx.max(x + 4.0), ty, w - 8.0);
                                }
                            }
                        }
                        _ => {
                            let pt2 = primitives::lp(&style.padding_top);
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
                                                let ch = lr.size.height
                                                    - pt2
                                                    - primitives::lp(&style.padding_bottom)
                                                    - bt2
                                                    - style.border.bottom.width;
                                                let lh = style.font_size * 1.4;
                                                let vy = if ch > lh { ty + (ch - lh) * 0.5 } else { ty };
                                                text::render_text(
                                                    dt,
                                                    &mut self.fonts,
                                                    &style,
                                                    &text,
                                                    tx,
                                                    vy,
                                                    aw,
                                                );
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
                    for c in &node.children {
                        self.paint_node(dt, styles, layout, form, c, px, py, caret_on);
                    }
                }
            }
            NodeType::Text(_) => {}
        }
    }

    fn get_or_load_image(&mut self, src: &str, target_w: u32, target_h: u32) -> Option<&LoadedImage> {
        let cache_key = format!("{}_{}_{}", src, target_w, target_h);
        if !self.image_cache.contains_key(&cache_key) {
            let loaded = load_image_file(src, target_w, target_h);
            if let Some(img) = loaded {
                self.image_cache.insert(cache_key.clone(), img);
            } else {
                return None;
            }
        }
        self.image_cache.get(&cache_key)
    }
}

fn load_image_file(src: &str, target_w: u32, target_h: u32) -> Option<LoadedImage> {
    let img = image::open(src).ok()?;
    let img_resized = img.resize(target_w, target_h, image::imageops::FilterType::Triangle);
    let img_rgba = img_resized.to_rgba8();
    let (width, height) = img_rgba.dimensions();
    let pixels: Vec<u32> = img_rgba
        .pixels()
        .map(|p| {
            let r = p[0] as u32;
            let g = p[1] as u32;
            let b = p[2] as u32;
            let a = p[3] as u32;
            let r = (r * a) / 255;
            let g = (g * a) / 255;
            let b = (b * a) / 255;
            (a << 24) | (r << 16) | (g << 8) | b
        })
        .collect();
    Some(LoadedImage {
        width: width as i32,
        height: height as i32,
        pixels,
    })
}
