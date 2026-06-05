use std::collections::HashMap;

#[derive(Clone)]
pub struct FormState {
    pub input_values: HashMap<String, String>,
    pub checked: HashMap<String, bool>,
    pub focused: Option<String>,
    pub cursor_pos: HashMap<String, usize>,
    pub sel_all: Option<String>,
}

impl FormState {
    pub fn new() -> Self {
        FormState { input_values: HashMap::new(), checked: HashMap::new(), focused: None, cursor_pos: HashMap::new(), sel_all: None }
    }

    pub fn get_value(&self, id: &str) -> &str {
        self.input_values.get(id).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn set_value(&mut self, id: &str, val: String) {
        self.input_values.insert(id.to_string(), val);
    }

    pub fn cursor(&self, id: &str) -> usize {
        self.cursor_pos.get(id).copied().unwrap_or(0)
    }

    pub fn set_cursor(&mut self, id: &str, pos: usize) {
        self.cursor_pos.insert(id.to_string(), pos);
    }

    pub fn is_checked(&self, id: &str) -> bool {
        self.checked.get(id).copied().unwrap_or(false)
    }

    pub fn toggle(&mut self, id: &str) {
        let v = self.checked.get(id).copied().unwrap_or(false);
        self.checked.insert(id.to_string(), !v);
    }

    pub fn focus(&mut self, id: Option<String>) {
        if id != self.focused { self.sel_all = None; }
        self.focused = id;
    }

    pub fn select_all(&mut self, id: &str) {
        self.sel_all = Some(id.to_string());
    }

    pub fn is_selected(&self, id: &str) -> bool {
        self.sel_all.as_ref().map_or(false, |s| s == id)
    }
}
