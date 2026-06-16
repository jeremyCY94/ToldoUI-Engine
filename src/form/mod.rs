pub mod actions;

use std::collections::HashMap;

#[derive(Clone)]
pub struct FormState {
    pub input_values: HashMap<String, String>,
    pub checked: HashMap<String, bool>,
    pub focused: Option<String>,
    pub cursor_pos: HashMap<String, usize>,
    pub selection: HashMap<String, (usize, usize)>, // (start, end)
    pub scroll_x: HashMap<String, f32>,
}

impl FormState {
    pub fn new() -> Self {
        FormState {
            input_values: HashMap::new(),
            checked: HashMap::new(),
            focused: None,
            cursor_pos: HashMap::new(),
            selection: HashMap::new(),
            scroll_x: HashMap::new(),
        }
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
        if id != self.focused {
            if let Some(ref prev) = self.focused {
                self.selection.remove(prev);
            }
        }
        self.focused = id;
    }

    pub fn select_all(&mut self, id: &str) {
        let val_len = self.get_value(id).chars().count();
        self.set_selection(id, 0, val_len);
    }

    pub fn set_selection(&mut self, id: &str, start: usize, end: usize) {
        self.selection.insert(id.to_string(), (start, end));
    }

    pub fn get_selection(&self, id: &str) -> Option<(usize, usize)> {
        self.selection.get(id).copied()
    }

    pub fn clear_selection(&mut self, id: &str) {
        self.selection.remove(id);
    }

    pub fn is_selected(&self, id: &str) -> bool {
        if let Some((start, end)) = self.selection.get(id) {
            start != end
        } else {
            false
        }
    }

    pub fn get_scroll_x(&self, id: &str) -> f32 {
        self.scroll_x.get(id).copied().unwrap_or(0.0)
    }

    pub fn set_scroll_x(&mut self, id: &str, val: f32) {
        self.scroll_x.insert(id.to_string(), val);
    }
}
