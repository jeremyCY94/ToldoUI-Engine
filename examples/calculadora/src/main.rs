use std::cell::RefCell;
use std::rc::Rc;
use winit::event_loop::EventLoop;
use toldo_ui_engine::core::app::App;
use toldo_ui_engine::render::overlay::{ModalState, ModalType};

struct CalculatorState {
    current_value: String,
    history_value: String,
    accumulated_value: f64,
    current_operator: Option<char>,
    start_new_number: bool,
}

impl CalculatorState {
    fn new() -> Self {
        CalculatorState {
            current_value: "0".to_string(),
            history_value: String::new(),
            accumulated_value: 0.0,
            current_operator: None,
            start_new_number: false,
        }
    }

    // Formatear un f64 para que quepa bien en el display y no tenga decimales innecesarios
    fn format_number(val: f64) -> String {
        if val.is_nan() {
            return "Error".to_string();
        }
        if val.is_infinite() {
            return "Infinito".to_string();
        }

        // Si es entero, lo mostramos sin decimales
        if val == val.trunc() {
            let s = format!("{:.0}", val);
            if s.len() > 14 {
                format!("{:e}", val)
            } else {
                s
            }
        } else {
            let s = format!("{}", val);
            if s.len() > 14 {
                // Intentar formatear con menos decimales
                let s_trunc = format!("{:.8}", val);
                // Quitar ceros a la derecha
                let clean = s_trunc.trim_end_matches('0').trim_end_matches('.').to_string();
                if clean.len() > 14 {
                    format!("{:e}", val)
                } else {
                    clean
                }
            } else {
                s
            }
        }
    }

    fn calculate(&mut self) -> Result<f64, &'static str> {
        let current: f64 = self.current_value.parse().unwrap_or(0.0);
        if let Some(op) = self.current_operator {
            match op {
                '+' => Ok(self.accumulated_value + current),
                '-' => Ok(self.accumulated_value - current),
                '*' => Ok(self.accumulated_value * current),
                '/' => {
                    if current == 0.0 {
                        Err("No se puede dividir por cero")
                    } else {
                        Ok(self.accumulated_value / current)
                    }
                }
                _ => Ok(current),
            }
        } else {
            Ok(current)
        }
    }
}

fn main() {
    let el = EventLoop::new().unwrap();
    let html = include_str!("calculator.html");
    let css = include_str!("calculator.css");

    let mut app = App::new("Calculadora Clásica de Windows")
        .with_initial_content(html, css)
        .with_size(340.0, 480.0);

    let state = Rc::new(RefCell::new(CalculatorState::new()));

    // Helper para actualizar el DOM con el estado actual de la calculadora
    let update_ui = {
        let state = state.clone();
        move |app: &mut App| {
            let st = state.borrow();
            app.rquery("#calc-display").set_text(&st.current_value);
            app.rquery("#calc-history").set_text(&st.history_value);
        }
    };

    // Registrar eventos para números (0-9)
    for i in 0..=9 {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        let id = format!("#btn-{}", i);
        app.rquery(&id).on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            let digit = i.to_string();
            if st.start_new_number {
                st.current_value = digit;
                st.start_new_number = false;
            } else {
                if st.current_value == "0" {
                    st.current_value = digit;
                } else {
                    st.current_value.push_str(&digit);
                }
            }
            drop(st);
            update_ui_clone(app);
        });
    }

    // Registrar punto decimal
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-dot").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            if st.start_new_number {
                st.current_value = "0.".to_string();
                st.start_new_number = false;
            } else if !st.current_value.contains('.') {
                st.current_value.push('.');
            }
            drop(st);
            update_ui_clone(app);
        });
    }

    // Registrar borrado completo C
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-c").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            *st = CalculatorState::new();
            drop(st);
            update_ui_clone(app);
        });
    }

    // Registrar borrado de entrada CE
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-ce").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            st.current_value = "0".to_string();
            st.start_new_number = false;
            drop(st);
            update_ui_clone(app);
        });
    }

    // Registrar retroceso / backspace
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-back").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            if st.start_new_number {
                // Si acabamos de hacer una operación, borrar limpia la pantalla
                st.current_value = "0".to_string();
            } else {
                st.current_value.pop();
                if st.current_value.is_empty() || st.current_value == "-" {
                    st.current_value = "0".to_string();
                }
            }
            drop(st);
            update_ui_clone(app);
        });
    }

    // Registrar cambio de signo ±
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-sign").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            if st.current_value != "0" && !st.current_value.is_empty() {
                if st.current_value.starts_with('-') {
                    st.current_value.remove(0);
                } else {
                    st.current_value.insert(0, '-');
                }
            }
            drop(st);
            update_ui_clone(app);
        });
    }

    // Registrar operadores (+, -, *, /)
    let ops = [('+', "#btn-add"), ('-', "#btn-sub"), ('*', "#btn-mul"), ('/', "#btn-div")];
    for (op, id) in ops {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery(id).on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            
            // Si ya hay un operador pendiente y no estamos listos para empezar un nuevo número, calculamos el resultado parcial
            if st.current_operator.is_some() && !st.start_new_number {
                match st.calculate() {
                    Ok(result) => {
                        st.accumulated_value = result;
                        st.current_value = CalculatorState::format_number(result);
                    }
                    Err(err) => {
                        app.modal = Some(ModalState {
                            title: "Error Matemático".to_string(),
                            message: err.to_string(),
                            modal_type: ModalType::Alert,
                            action: "calc_error".to_string(),
                        });
                        *st = CalculatorState::new();
                        drop(st);
                        update_ui_clone(app);
                        return;
                    }
                }
            } else {
                st.accumulated_value = st.current_value.parse().unwrap_or(0.0);
            }

            st.current_operator = Some(op);
            st.history_value = format!("{} {}", CalculatorState::format_number(st.accumulated_value), op);
            st.start_new_number = true;
            drop(st);
            update_ui_clone(app);
        });
    }

    // Registrar botón igual =
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-equal").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            if let Some(op) = st.current_operator {
                let current_val = st.current_value.clone();
                match st.calculate() {
                    Ok(result) => {
                        st.history_value = format!(
                            "{} {} {} =",
                            CalculatorState::format_number(st.accumulated_value),
                            op,
                            current_val
                        );
                        st.current_value = CalculatorState::format_number(result);
                        st.accumulated_value = result;
                        st.current_operator = None;
                        st.start_new_number = true;
                    }
                    Err(err) => {
                        app.modal = Some(ModalState {
                            title: "Error de Operación".to_string(),
                            message: err.to_string(),
                            modal_type: ModalType::Alert,
                            action: "calc_error".to_string(),
                        });
                        *st = CalculatorState::new();
                    }
                }
            }
            drop(st);
            update_ui_clone(app);
        });
    }

    // Registrar operaciones unarias (% y funciones avanzadas)
    // Para porcentaje %:
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-percent").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            let current: f64 = st.current_value.parse().unwrap_or(0.0);
            let result = current / 100.0;
            st.current_value = CalculatorState::format_number(result);
            st.start_new_number = true;
            drop(st);
            update_ui_clone(app);
        });
    }

    // Inverso (1/x)
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-inv").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            let current: f64 = st.current_value.parse().unwrap_or(0.0);
            if current == 0.0 {
                app.modal = Some(ModalState {
                    title: "División por Cero".to_string(),
                    message: "No se puede calcular el inverso de cero.".to_string(),
                    modal_type: ModalType::Alert,
                    action: "calc_error".to_string(),
                });
                *st = CalculatorState::new();
            } else {
                let result = 1.0 / current;
                st.history_value = format!("1/({})", CalculatorState::format_number(current));
                st.current_value = CalculatorState::format_number(result);
                st.start_new_number = true;
            }
            drop(st);
            update_ui_clone(app);
        });
    }

    // Cuadrado (x²)
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-sqr").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            let current: f64 = st.current_value.parse().unwrap_or(0.0);
            let result = current * current;
            st.history_value = format!("sqr({})", CalculatorState::format_number(current));
            st.current_value = CalculatorState::format_number(result);
            st.start_new_number = true;
            drop(st);
            update_ui_clone(app);
        });
    }

    // Raíz cuadrada (√x)
    {
        let state_clone = state.clone();
        let update_ui_clone = update_ui.clone();
        app.rquery("#btn-sqrt").on_click(move |app, _node| {
            let mut st = state_clone.borrow_mut();
            let current: f64 = st.current_value.parse().unwrap_or(0.0);
            if current < 0.0 {
                app.modal = Some(ModalState {
                    title: "Entrada Inválida".to_string(),
                    message: "No se puede calcular la raíz cuadrada de un número negativo.".to_string(),
                    modal_type: ModalType::Alert,
                    action: "calc_error".to_string(),
                });
                *st = CalculatorState::new();
            } else {
                let result = current.sqrt();
                st.history_value = format!("sqrt({})", CalculatorState::format_number(current));
                st.current_value = CalculatorState::format_number(result);
                st.start_new_number = true;
            }
            drop(st);
            update_ui_clone(app);
        });
    }

    el.run_app(&mut app).unwrap();
}
