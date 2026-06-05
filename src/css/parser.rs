use std::collections::HashMap;

use crate::css::{Stylesheet, Rule, SelectorComponent, Combinator, CompoundSelector};

pub fn parse_stylesheet(css_text: &str) -> Stylesheet {
    let mut rules = Vec::new();
    let chars: Vec<char> = css_text.chars().collect();
    let len = chars.len();
    let mut pos = 0;

    while pos < len {
        pos = skip_ws(&chars, pos);
        if pos >= len { break; }

        if chars[pos] == '@' { pos = skip_at_rule(&chars, pos); continue; }

        let sel_start = pos;
        // Find the '{' that begins the declaration block
        let open_pos = {
            let mut p = pos;
            while p < len && chars[p] != '{' { p += 1; }
            p
        };
        if open_pos >= len { break; }
        let sel_text: String = chars[sel_start..open_pos].iter().collect();
        let sel_text = sel_text.trim();

        // Find the matching '}'
        pos = skip_balanced(&chars, open_pos, '{', '}');
        if pos >= len { break; }
        let decl_text: String = chars[open_pos + 1..pos].iter().collect();
        pos += 1;

        let selectors = parse_selector_list(sel_text);
        let declarations = parse_declarations(&decl_text);
        if !selectors.is_empty() {
            rules.push(Rule { selectors, declarations });
        }
    }
    Stylesheet { rules }
}

fn parse_selector_list(text: &str) -> Vec<Vec<SelectorComponent>> {
    text.split(',').map(|s| parse_selector(s.trim())).filter(|s: &Vec<SelectorComponent>| !s.is_empty()).collect()
}

fn push_comp(comps: &mut Vec<SelectorComponent>, tag: &mut Option<String>, id: &mut Option<String>,
             classes: &mut Vec<String>, attrs: &mut Vec<(String, Option<String>)>, pseudo: &mut Vec<String>) {
    if tag.is_some() || id.is_some() || !classes.is_empty() || !attrs.is_empty() || !pseudo.is_empty() {
        let mut cls = Vec::new(); std::mem::swap(classes, &mut cls);
        let mut at = Vec::new(); std::mem::swap(attrs, &mut at);
        let mut ps = Vec::new(); std::mem::swap(pseudo, &mut ps);
        comps.push(SelectorComponent::Compound(CompoundSelector {
            tag: tag.take(), id: id.take(), classes: cls, attributes: at, pseudo_classes: ps,
        }));
    }
}

fn parse_selector(text: &str) -> Vec<SelectorComponent> {
    if text.is_empty() { return vec![]; }
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut pos = 0;
    let mut comps = Vec::new();
    let mut tag: Option<String> = None;
    let mut id: Option<String> = None;
    let mut classes: Vec<String> = Vec::new();
    let mut attrs: Vec<(String, Option<String>)> = Vec::new();
    let mut pseudo: Vec<String> = Vec::new();
    let mut has_universal = false;

    while pos < len {
        match chars[pos] {
            c if c.is_whitespace() => {
                push_comp(&mut comps, &mut tag, &mut id, &mut classes, &mut attrs, &mut pseudo);
                let mut sp = false;
                while pos < len && chars[pos].is_whitespace() { sp = true; pos += 1; }
                if sp && pos < len && chars[pos] != '>' && chars[pos] != '+' && chars[pos] != '~' {
                    comps.push(SelectorComponent::Combinator(Combinator::Descendant));
                }
            }
            '>' => { push_comp(&mut comps, &mut tag, &mut id, &mut classes, &mut attrs, &mut pseudo);
                     comps.push(SelectorComponent::Combinator(Combinator::Child)); pos += 1; }
            '+' | '~' => { push_comp(&mut comps, &mut tag, &mut id, &mut classes, &mut attrs, &mut pseudo);
                           comps.push(SelectorComponent::Combinator(Combinator::Descendant)); pos += 1; }
            '#' => { pos += 1; let mut s = String::new();
                     while pos < len && is_id_char(chars[pos]) { s.push(chars[pos]); pos += 1; } id = Some(s); }
            '.' => { pos += 1; let mut s = String::new();
                     while pos < len && is_id_char(chars[pos]) { s.push(chars[pos]); pos += 1; } classes.push(s); }
            '[' => { pos += 1; let mut n = String::new();
                     while pos < len && chars[pos] != ']' && chars[pos] != '=' { if !chars[pos].is_whitespace() { n.push(chars[pos]); } pos += 1; }
                     let mut v = None;
                     if pos < len && chars[pos] == '=' { pos += 1; let mut val = String::new();
                         if pos < len && (chars[pos] == '"' || chars[pos] == '\'') { let q = chars[pos]; pos += 1;
                             while pos < len && chars[pos] != q { val.push(chars[pos]); pos += 1; } if pos < len { pos += 1; }
                         } else { while pos < len && chars[pos] != ']' && !chars[pos].is_whitespace() { val.push(chars[pos]); pos += 1; } }
                         v = Some(val); }
                     while pos < len && chars[pos] != ']' { pos += 1; } if pos < len { pos += 1; }
                     attrs.push((n, v)); }
            ':' => { pos += 1; let mut s = String::new();
                     while pos < len && is_id_char(chars[pos]) { s.push(chars[pos]); pos += 1; } pseudo.push(s); }
            '*' => { pos += 1; has_universal = true; }
            c if c.is_alphabetic() || c == '_' => {
                let mut n = String::new();
                while pos < len && is_id_char(chars[pos]) { n.push(chars[pos]); pos += 1; }
                if tag.is_none() && id.is_none() && classes.is_empty() && attrs.is_empty() {
                    tag = Some(n.to_lowercase());
                }
            }
            _ => { pos += 1; }
        }
    }
    push_comp(&mut comps, &mut tag, &mut id, &mut classes, &mut attrs, &mut pseudo);
    if has_universal && comps.is_empty() {
        comps.push(SelectorComponent::Compound(CompoundSelector {
            tag: None, id: None, classes: vec![], attributes: vec![], pseudo_classes: vec![],
        }));
    }
    comps
}

fn is_id_char(c: char) -> bool { c.is_alphanumeric() || c == '_' || c == '-' || c > '\u{007F}' }

fn skip_ws(chars: &[char], mut pos: usize) -> usize {
    let len = chars.len();
    while pos < len {
        if chars[pos].is_whitespace() { pos += 1; }
        else if pos + 1 < len && chars[pos] == '/' && chars[pos + 1] == '*' {
            pos += 2;
            while pos + 1 < len && !(chars[pos] == '*' && chars[pos + 1] == '/') { pos += 1; }
            if pos + 1 < len { pos += 2; }
        } else { break; }
    }
    pos
}

fn skip_balanced(chars: &[char], mut pos: usize, open: char, close: char) -> usize {
    let len = chars.len();
    let mut depth = 0;
    let mut first = true;
    while pos < len {
        if first && chars[pos] == open { depth = 1; first = false; pos += 1; }
        else if !first {
            if chars[pos] == open { depth += 1; }
            else if chars[pos] == close { depth -= 1; if depth == 0 { return pos; } }
            else if chars[pos] == '"' || chars[pos] == '\'' {
                let q = chars[pos]; pos += 1;
                while pos < len && chars[pos] != q { if chars[pos] == '\\' { pos += 1; } pos += 1; }
            }
            pos += 1;
        } else { pos += 1; }
    }
    len
}

fn skip_at_rule(chars: &[char], mut pos: usize) -> usize {
    while pos < chars.len() && chars[pos] != '{' && chars[pos] != ';' { pos += 1; }
    if pos < chars.len() && chars[pos] == '{' {
        pos = skip_balanced(chars, pos, '{', '}');
        if pos < chars.len() { pos += 1; }
    } else if pos < chars.len() { pos += 1; }
    pos
}

fn parse_declarations(text: &str) -> HashMap<String, String> {
    let mut decls = HashMap::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut pos = 0;

    while pos < len {
        pos = skip_ws(&chars, pos);
        if pos >= len { break; }

        let ps = pos;
        while pos < len && chars[pos] != ':' { pos += 1; }
        if pos >= len { break; }
        let pn: String = chars[ps..pos].iter().collect();
        pos += 1;

        pos = skip_ws(&chars, pos);
        let vs = pos;
        while pos < len && chars[pos] != ';' && chars[pos] != '}' {
            if chars[pos] == '"' || chars[pos] == '\'' {
                let q = chars[pos]; pos += 1;
                while pos < len && chars[pos] != q { if chars[pos] == '\\' { pos += 1; } pos += 1; }
                if pos < len { pos += 1; }
            } else if chars[pos] == '(' {
                let mut d = 1; pos += 1;
                while pos < len && d > 0 {
                    if chars[pos] == '(' { d += 1; } else if chars[pos] == ')' { d -= 1; }
                    pos += 1;
                }
            } else { pos += 1; }
        }
        let pv: String = chars[vs..pos].iter().collect();
        let pn = pn.trim().to_lowercase();
        let pv = pv.trim().to_string();
        if !pn.is_empty() && !pv.is_empty() { decls.insert(pn, pv); }
        if pos < len && chars[pos] == ';' { pos += 1; }
    }
    decls
}
