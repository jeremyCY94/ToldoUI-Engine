use super::FormState;

pub fn delete_selected_text(form: &mut FormState, id: &str) -> bool {
    if let Some((start, end)) = form.get_selection(id) {
        if start != end {
            let s_min = start.min(end);
            let s_max = start.max(end);
            let val = form.get_value(id);
            let chars: Vec<char> = val.chars().collect();
            if s_min < chars.len() {
                let mut new_chars = Vec::new();
                for i in 0..chars.len() {
                    if i < s_min || i >= s_max {
                        new_chars.push(chars[i]);
                    }
                }
                let new_val: String = new_chars.into_iter().collect();
                form.set_value(id, new_val);
                form.set_cursor(id, s_min);
            } else if chars.is_empty() {
                form.set_value(id, String::new());
                form.set_cursor(id, 0);
            }
            form.clear_selection(id);
            return true;
        }
    }
    false
}

pub fn get_selected_text(form: &FormState, id: &str) -> Option<String> {
    if let Some((start, end)) = form.get_selection(id) {
        if start != end {
            let val = form.get_value(id);
            let s_min = start.min(end);
            let s_max = start.max(end);
            let chars: Vec<char> = val.chars().collect();
            if s_min < chars.len() {
                let selected_chars: Vec<char> = chars[s_min..s_max.min(chars.len())].to_vec();
                return Some(selected_chars.into_iter().collect());
            }
        }
    }
    None
}

pub fn insert_text(form: &mut FormState, id: &str, text: &str) {
    delete_selected_text(form, id);
    
    let val = form.get_value(id);
    let len = val.chars().count();
    let pos = form.cursor(id).min(len);
    let mut chars: Vec<char> = val.chars().collect();
    
    for (i, c) in text.chars().enumerate() {
        chars.insert(pos + i, c);
    }
    
    let new_val: String = chars.into_iter().collect();
    form.set_value(id, new_val);
    form.set_cursor(id, pos + text.chars().count());
}
