use toldo_ui_engine::core::app::App;
use winit::event_loop::EventLoop;
use toldo_ui_engine::render::overlay::{ModalState, ModalType};

fn main() {
    let el = EventLoop::new().unwrap();
    let html = include_str!("../examples/simple.html");
    let css = include_str!("../examples/simple.css");
    let mut app = App::new("ToldoUI-Engine")
        .with_initial_content(html, css)
        .with_size(1024.0, 768.0);

    // Registrar eventos utilizando la API estilo RQuery de manera súper sencilla
    app.rquery("#btn-enviar").on_click(|app, _node| {
        app.modal = Some(ModalState {
            title: "Confirmar Envío".to_string(),
            message: "¿Estás seguro de que deseas enviar el formulario? Esta acción iniciará un procesamiento de datos.".to_string(),
            modal_type: ModalType::Confirm,
            action: "confirm_submit".to_string(),
        });
    });

    app.rquery("#btn-default").on_click(|app, _node| {
        // Manipulación dinámica del DOM
        app.rquery("#header h1").set_text("¡RQuery en Rust!");
        app.rquery("#header p").set_text("El DOM ha sido modificado dinámicamente con estilos de alto nivel.");
        // Modificación dinámica de estilos
        app.rquery("#header").set_attr(
            "style",
            "display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 30px 20px; background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 100%);"
        );
    });

    app.rquery("#btn-hover").on_click(|app, node| {
        let label = node.children_text().trim().to_string();
        app.modal = Some(ModalState {
            title: "Botón con Hover Presionado".to_string(),
            message: format!(
                "Has presionado el botón: \"{}\". Este evento de clic y la obtención del texto del elemento se manejaron mediante selectores RQuery.",
                label
            ),
            modal_type: ModalType::Alert,
            action: "alert_demo".to_string(),
        });
    });

    // --- PLAYGROUND RQUERY EVENT HANDLERS ---

    // 1. Class & Style Manipulation
    app.rquery("#btn-add-class").on_click(|app, _node| {
        app.rquery("#playground-target").add_class("medium");
    });

    app.rquery("#btn-remove-class").on_click(|app, _node| {
        app.rquery("#playground-target").remove_class("medium");
    });

    app.rquery("#btn-toggle-class").on_click(|app, _node| {
        app.rquery("#playground-target").toggle_class("medium");
    });

    app.rquery("#btn-css-color").on_click(|app, _node| {
        app.rquery("#playground-target").css("background-color", "#4f46e5");
    });

    app.rquery("#btn-hide").on_click(|app, _node| {
        app.rquery("#playground-target").hide();
    });

    app.rquery("#btn-show").on_click(|app, _node| {
        app.rquery("#playground-target").show();
    });

    // 2. DOM Insertions & Deletions
    app.rquery("#btn-append").on_click(|app, _node| {
        app.rquery("#playground-target").append("<span style='color: #ffeb3b; font-size: 13px; margin-left: 6px;'>(Append)</span>");
    });

    app.rquery("#btn-prepend").on_click(|app, _node| {
        app.rquery("#playground-target").prepend("<span style='color: #00e676; font-size: 13px; margin-right: 6px;'>(Prepend)</span>");
    });

    app.rquery("#btn-empty").on_click(|app, _node| {
        app.rquery("#playground-target").empty();
    });

    app.rquery("#btn-remove").on_click(|app, _node| {
        app.rquery("#playground-target").remove();
    });

    app.rquery("#btn-reset-html").on_click(|app, _node| {
        app.rquery("#playground-wrapper").empty().append(
            "<div id='playground-target' class='box box-1' style='height: 60px; min-height: 60px; display: flex; align-items: center; justify-content: center; font-weight: bold; border-radius: 6px;'>Playground Target</div>"
        );
    });

    // 3. Form Values & States
    app.rquery("#btn-get-val").on_click(|app, _node| {
        let val = app.rquery("#playground-input").val();
        app.modal = Some(ModalState {
            title: "Valor de Input".to_string(),
            message: format!("El valor actual del campo de texto es: \"{}\"", val),
            modal_type: ModalType::Alert,
            action: "get_val_alert".to_string(),
        });
    });

    app.rquery("#btn-set-val").on_click(|app, _node| {
        app.rquery("#playground-input").set_val("Hola RQuery");
    });

    app.rquery("#btn-get-checked").on_click(|app, _node| {
        let checked = app.rquery("#playground-checkbox").is_checked();
        app.modal = Some(ModalState {
            title: "Estado de Checkbox".to_string(),
            message: format!("¿El checkbox está marcado?: {}", if checked { "Sí" } else { "No" }),
            modal_type: ModalType::Alert,
            action: "get_checked_alert".to_string(),
        });
    });

    app.rquery("#btn-set-checked-true").on_click(|app, _node| {
        app.rquery("#playground-checkbox").set_checked(true);
    });

    app.rquery("#btn-set-checked-false").on_click(|app, _node| {
        app.rquery("#playground-checkbox").set_checked(false);
    });

    el.run_app(&mut app).unwrap();
}
