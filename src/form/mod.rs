pub mod actions;

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSection {
    Day,
    Month,
    Year,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeSection {
    Hour,
    Minute,
    Second,
}

#[derive(Clone)]
pub struct FormState {
    pub input_values: HashMap<String, String>,
    pub checked: HashMap<String, bool>,
    pub focused: Option<String>,
    pub cursor_pos: HashMap<String, usize>,
    pub selection: HashMap<String, (usize, usize)>, // (start, end)
    pub scroll_x: HashMap<String, f32>,
    pub dropdown_scroll_y: HashMap<String, f32>,
    pub date_active_section: HashMap<String, DateSection>,
    pub date_years_range: HashMap<String, (i32, i32)>,
    pub time_active_section: HashMap<String, TimeSection>,
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
            dropdown_scroll_y: HashMap::new(),
            date_active_section: HashMap::new(),
            date_years_range: HashMap::new(),
            time_active_section: HashMap::new(),
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

    pub fn get_dropdown_scroll_y(&self, id: &str) -> f32 {
        self.dropdown_scroll_y.get(id).copied().unwrap_or(0.0)
    }

    pub fn set_dropdown_scroll_y(&mut self, id: &str, val: f32) {
        self.dropdown_scroll_y.insert(id.to_string(), val);
    }

    pub fn get_date_active_section(&self, id: &str) -> Option<DateSection> {
        self.date_active_section.get(id).copied()
    }

    pub fn set_date_active_section(&mut self, id: &str, sec: DateSection) {
        self.date_active_section.insert(id.to_string(), sec);
    }

    pub fn get_date_years_range(&self, id: &str) -> (i32, i32) {
        self.date_years_range.get(id).copied().unwrap_or_else(|| {
            let current = get_current_year();
            (current - 10, current + 10)
        })
    }

    pub fn set_date_years_range(&mut self, id: &str, range: (i32, i32)) {
        self.date_years_range.insert(id.to_string(), range);
    }

    pub fn get_time_active_section(&self, id: &str) -> Option<TimeSection> {
        self.time_active_section.get(id).copied()
    }

    pub fn set_time_active_section(&mut self, id: &str, sec: TimeSection) {
        self.time_active_section.insert(id.to_string(), sec);
    }
}

pub fn get_current_year() -> i32 {
    if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        let secs = duration.as_secs();
        let days = secs / 86400;
        let mut year = 1970;
        let mut days_left = days;
        loop {
            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let days_in_year = if is_leap { 366 } else { 365 };
            if days_left < days_in_year {
                break;
            }
            days_left -= days_in_year;
            year += 1;
        }
        year as i32
    } else {
        2026
    }
}
