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
    pub date_picker_open: HashMap<String, bool>,
    pub date_picker_month: HashMap<String, u32>,
    pub date_picker_year: HashMap<String, i32>,
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
            date_picker_open: HashMap::new(),
            date_picker_month: HashMap::new(),
            date_picker_year: HashMap::new(),
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

    pub fn is_date_picker_open(&self, id: &str) -> bool {
        self.date_picker_open.get(id).copied().unwrap_or(false)
    }

    pub fn set_date_picker_open(&mut self, id: &str, open: bool) {
        self.date_picker_open.insert(id.to_string(), open);
    }

    pub fn get_date_picker_month_year(&self, id: &str) -> (u32, i32) {
        let (default_m, default_y) = get_current_month_year();
        let m = self.date_picker_month.get(id).copied().unwrap_or(default_m);
        let y = self.date_picker_year.get(id).copied().unwrap_or(default_y);
        (m, y)
    }

    pub fn set_date_picker_month_year(&mut self, id: &str, m: u32, y: i32) {
        self.date_picker_month.insert(id.to_string(), m);
        self.date_picker_year.insert(id.to_string(), y);
    }
}

pub fn get_current_year() -> i32 {
    get_current_month_year().1
}

pub fn get_current_month_year() -> (u32, i32) {
    if let Ok(duration) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        let secs = duration.as_secs();
        let mut days_left = secs / 86400;
        let mut year = 1970;
        loop {
            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let days_in_year = if is_leap { 366 } else { 365 };
            if days_left < days_in_year {
                break;
            }
            days_left -= days_in_year;
            year += 1;
        }
        
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let month_days = if is_leap {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        
        let mut month = 1;
        for &days in &month_days {
            if days_left < days as u64 {
                break;
            }
            days_left -= days as u64;
            month += 1;
        }
        
        (month, year as i32)
    } else {
        (6, 2026)
    }
}

pub fn parse_date_value(val: &str, format: &str) -> Option<(u32, u32, i32)> {
    let format_lower = format.to_lowercase();
    let chars: Vec<char> = val.chars().collect();
    if format_lower == "yyyy-mm-dd" {
        if chars.len() < 10 {
            return None;
        }
        let y: i32 = chars[0..4].iter().collect::<String>().parse().ok()?;
        let m: u32 = chars[5..7].iter().collect::<String>().parse().ok()?;
        let d: u32 = chars[8..10].iter().collect::<String>().parse().ok()?;
        Some((d, m, y))
    } else {
        // dd/MM/yyyy
        if chars.len() < 10 {
            return None;
        }
        let d: u32 = chars[0..2].iter().collect::<String>().parse().ok()?;
        let m: u32 = chars[3..5].iter().collect::<String>().parse().ok()?;
        let y: i32 = chars[6..10].iter().collect::<String>().parse().ok()?;
        Some((d, m, y))
    }
}

pub fn day_of_week(y: i32, m: u32, d: u32) -> u32 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = y;
    if m < 3 {
        y -= 1;
    }
    let res = (y + y / 4 - y / 100 + y / 400 + t[(m - 1) as usize] + d as i32) % 7;
    // Handle potential negative modulus just in case, though Sakamoto yields >=0
    let res = if res < 0 { res + 7 } else { res };
    res as u32
}

pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let is_leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
            if is_leap { 29 } else { 28 }
        }
        _ => 30,
    }
}
