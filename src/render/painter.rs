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

// Re-export methods used by input.rs and app.rs to keep compatibility
pub use super::text::{index_at_x, x_at_index};
pub use super::inputs::textarea_index_at_point;

pub struct Painter {
    fonts: HashMap<String, Font<'static>>,
}

impl Painter {
    pub fn new() -> Self {
        Painter {
            fonts: HashMap::new(),
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
        spinner_angle: f32,
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

        if loading {
            overlay::paint_loading_overlay(dt, vw, vh, spinner_angle);
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
}
