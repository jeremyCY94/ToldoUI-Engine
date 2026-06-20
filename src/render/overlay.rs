use std::collections::HashMap;
use raqote::{DrawTarget, SolidSource, Source, DrawOptions, PathBuilder, StrokeStyle, LineCap, LineJoin};
use rusttype::{Font, Scale};

#[derive(Clone, Debug, PartialEq)]
pub enum ModalType {
    Alert,
    Confirm,
}

#[derive(Clone, Debug)]
pub struct ModalState {
    pub title: String,
    pub message: String,
    pub modal_type: ModalType,
    pub action: String,
}

pub fn paint_loading_overlay(dt: &mut DrawTarget, vw: f32, vh: f32, spinner_angle: f32) {
    // 1. Fondo de la página en un gris muy claro (Slate-50)
    let bg_color = SolidSource::from_unpremultiplied_argb(255, 248, 250, 252);
    dt.fill_rect(0.0, 0.0, vw, vh, &Source::Solid(bg_color), &DrawOptions::new());

    // 2. Animación pulse basada en seno (opacidad oscila entre 0.35 y 0.65)
    let pulse = 0.35 + 0.30 * (spinner_angle * 1.5).sin().abs();
    let alpha = (pulse * 255.0) as u8;

    // Colores para los elementos del Skeleton
    let skeleton_color = Source::Solid(SolidSource::from_unpremultiplied_argb(alpha, 226, 232, 240)); // Slate-200
    let skeleton_color_dark = Source::Solid(SolidSource::from_unpremultiplied_argb(alpha, 203, 213, 225)); // Slate-300

    // 3. NAVBAR SUPERIOR MINIMALISTA
    let nav_bg = SolidSource::from_unpremultiplied_argb(255, 255, 255, 255);
    dt.fill_rect(0.0, 0.0, vw, 56.0, &Source::Solid(nav_bg), &DrawOptions::new());
    let nav_border = SolidSource::from_unpremultiplied_argb(255, 241, 245, 249);
    dt.fill_rect(0.0, 55.0, vw, 1.0, &Source::Solid(nav_border), &DrawOptions::new());

    // 4. ELEMENTOS DE CONTENIDO LIGEROS
    // Título principal
    let mut pb_title = PathBuilder::new();
    draw_rounded_rect_path(&mut pb_title, 40.0, 88.0, 220.0, 24.0, 4.0);
    dt.fill(&pb_title.finish(), &skeleton_color_dark, &DrawOptions::new());

    // Barra de texto 1
    let mut pb_line1 = PathBuilder::new();
    draw_rounded_rect_path(&mut pb_line1, 40.0, 130.0, vw - 80.0, 12.0, 3.0);
    dt.fill(&pb_line1.finish(), &skeleton_color, &DrawOptions::new());

    // Barra de texto 2
    let mut pb_line2 = PathBuilder::new();
    draw_rounded_rect_path(&mut pb_line2, 40.0, 150.0, (vw - 80.0) * 0.8, 12.0, 3.0);
    dt.fill(&pb_line2.finish(), &skeleton_color, &DrawOptions::new());

    // Bloque de contenido principal grande
    let mut pb_panel = PathBuilder::new();
    draw_rounded_rect_path(&mut pb_panel, 40.0, 186.0, vw - 80.0, (vh - 226.0).max(100.0), 8.0);
    dt.fill(&pb_panel.finish(), &skeleton_color, &DrawOptions::new());
}

pub fn paint_modal_overlay(
    dt: &mut DrawTarget,
    fonts: &mut HashMap<String, Font<'static>>,
    modal: &ModalState,
    vw: f32,
    vh: f32,
    mouse_x: f32,
    mouse_y: f32,
) {
    // Fondo oscuro translúcido
    let overlay_color = SolidSource::from_unpremultiplied_argb(160, 15, 23, 42);
    dt.fill_rect(0.0, 0.0, vw, vh, &Source::Solid(overlay_color), &DrawOptions::new());

    // Dimensiones del modal
    let mw = 420.0;
    let mh = 220.0;
    let mx = (vw - mw) / 2.0;
    let my = (vh - mh) / 2.0;

    // Sombra del modal
    let shadow_color = SolidSource::from_unpremultiplied_argb(40, 0, 0, 0);
    let mut pb_shadow = PathBuilder::new();
    draw_rounded_rect_path(&mut pb_shadow, mx + 4.0, my + 6.0, mw, mh, 14.0);
    dt.fill(&pb_shadow.finish(), &Source::Solid(shadow_color), &DrawOptions::new());

    // Fondo de la tarjeta blanca del modal
    let card_bg = SolidSource::from_unpremultiplied_argb(255, 255, 255, 255);
    let mut pb_card = PathBuilder::new();
    draw_rounded_rect_path(&mut pb_card, mx, my, mw, mh, 12.0);
    dt.fill(&pb_card.finish(), &Source::Solid(card_bg), &DrawOptions::new());

    // Borde fino Slate-200
    let border_color = SolidSource::from_unpremultiplied_argb(255, 226, 232, 240);
    let stroke_style = StrokeStyle {
        width: 1.0,
        cap: LineCap::Butt,
        join: LineJoin::Miter,
        miter_limit: 10.0,
        dash_array: Vec::new(),
        dash_offset: 0.0,
    };
    let mut pb_border = PathBuilder::new();
    draw_rounded_rect_path(&mut pb_border, mx, my, mw, mh, 12.0);
    dt.stroke(&pb_border.finish(), &Source::Solid(border_color), &stroke_style, &DrawOptions::new());

    // Cargar fuentes
    let title_font = super::text::load_font(fonts, "Arial", 700).unwrap(); // Negrita
    let body_font = super::text::load_font(fonts, "Arial", 400).unwrap();  // Normal

    // Renderizar Título
    let scale_title = Scale::uniform(18.0);
    let title_color = SolidSource::from_unpremultiplied_argb(255, 15, 23, 42); // slate-900
    super::text::draw_text_line(
        dt,
        &title_font,
        scale_title,
        &modal.title,
        mx + 24.0,
        my + 28.0 + title_font.v_metrics(scale_title).ascent,
        title_color,
    );

    // Renderizar Mensaje (con saltos de línea automáticos)
    let body_style = crate::style::ComputedStyle {
        font_size: 14.0,
        font_family: "Arial".to_string(),
        font_weight: 400,
        color: crate::style::Color::new(71, 85, 105, 255), // slate-600
        ..Default::default()
    };
    let body_color = SolidSource::from_unpremultiplied_argb(255, 71, 85, 105);
    super::text::render_text_simple(
        dt,
        fonts,
        &body_style,
        &modal.message,
        mx + 24.0,
        my + 68.0,
        mw - 48.0,
        body_color,
    );

    // Posición y dimensiones de botones
    let btn_w = 100.0;
    let btn_h = 36.0;
    let btn_y = my + mh - 24.0 - btn_h;
    let scale_btn = Scale::uniform(14.0);

    let accept_x = mx + mw - 24.0 - btn_w;

    // Dibujar Botón Aceptar
    let is_accept_hovered = mouse_x >= accept_x && mouse_x < accept_x + btn_w && mouse_y >= btn_y && mouse_y < btn_y + btn_h;
    let accept_bg = if is_accept_hovered {
        SolidSource::from_unpremultiplied_argb(255, 79, 70, 229) // Indigo-600
    } else {
        SolidSource::from_unpremultiplied_argb(255, 99, 102, 241) // Indigo-500
    };
    let mut pb_accept = PathBuilder::new();
    draw_rounded_rect_path(&mut pb_accept, accept_x, btn_y, btn_w, btn_h, 6.0);
    dt.fill(&pb_accept.finish(), &Source::Solid(accept_bg), &DrawOptions::new());

    let accept_label = "Aceptar";
    let accept_lw = super::text::mw(&body_font, scale_btn, accept_label);
    let accept_tx = accept_x + (btn_w - accept_lw) / 2.0;
    let accept_ty = btn_y + (btn_h - 14.0) / 2.0;
    super::text::draw_text_line(
        dt,
        &title_font,
        scale_btn,
        accept_label,
        accept_tx,
        accept_ty + title_font.v_metrics(scale_btn).ascent - 1.0,
        SolidSource::from_unpremultiplied_argb(255, 255, 255, 255),
    );

    // Si es tipo confirmación, dibujar también Botón Cancelar
    if modal.modal_type == ModalType::Confirm {
        let cancel_x = accept_x - 12.0 - btn_w;
        let is_cancel_hovered = mouse_x >= cancel_x && mouse_x < cancel_x + btn_w && mouse_y >= btn_y && mouse_y < btn_y + btn_h;
        let cancel_bg = if is_cancel_hovered {
            SolidSource::from_unpremultiplied_argb(255, 226, 232, 240) // Slate-200
        } else {
            SolidSource::from_unpremultiplied_argb(255, 241, 245, 249) // Slate-100
        };
        let mut pb_cancel = PathBuilder::new();
        draw_rounded_rect_path(&mut pb_cancel, cancel_x, btn_y, btn_w, btn_h, 6.0);
        dt.fill(&pb_cancel.finish(), &Source::Solid(cancel_bg), &DrawOptions::new());

        let cancel_label = "Cancelar";
        let cancel_lw = super::text::mw(&body_font, scale_btn, cancel_label);
        let cancel_tx = cancel_x + (btn_w - cancel_lw) / 2.0;
        let cancel_ty = btn_y + (btn_h - 14.0) / 2.0;
        super::text::draw_text_line(
            dt,
            &body_font,
            scale_btn,
            cancel_label,
            cancel_tx,
            cancel_ty + body_font.v_metrics(scale_btn).ascent - 1.0,
            SolidSource::from_unpremultiplied_argb(255, 51, 65, 85), // Slate-700
        );
    }
}

#[allow(dead_code)]
fn draw_circle_path(cx: f32, cy: f32, r: f32) -> raqote::Path {
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

fn draw_rounded_rect_path(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
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
}
