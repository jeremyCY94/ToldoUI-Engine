use std::collections::HashMap;

use crate::dom::Node;

pub mod parser;

#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Vec<SelectorComponent>>,
    pub declarations: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum SelectorComponent {
    Compound(CompoundSelector),
    Combinator(Combinator),
}

#[derive(Debug, Clone, Copy)]
pub enum Combinator {
    Descendant,
    Child,
}

#[derive(Debug, Clone)]
pub struct CompoundSelector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: Vec<(String, Option<String>)>,
    pub pseudo_classes: Vec<String>,
}

impl CompoundSelector {
    pub fn specificity(&self) -> (u32, u32, u32) {
        let id = if self.id.is_some() { 1 } else { 0 };
        let class = (self.classes.len() + self.attributes.len() + self.pseudo_classes.len()) as u32;
        let tag = if self.tag.is_some() { 1 } else { 0 };
        (id, class, tag)
    }

    pub fn matches(&self, node: &Node) -> bool {
        let edata = match node.element_data() {
            Some(d) => d,
            None => return false,
        };

        if let Some(ref tag) = self.tag {
            if !edata.tag_name.eq_ignore_ascii_case(tag) { return false; }
        }
        if let Some(ref id) = self.id {
            match edata.attributes.get("id") {
                Some(v) if v == id => {}
                _ => return false,
            }
        }
        for class in &self.classes {
            if !node.has_class(class) { return false; }
        }
        for (attr_name, attr_value) in &self.attributes {
            match edata.attributes.get(attr_name) {
                Some(val) => {
                    if let Some(expected) = attr_value {
                        if val != expected { return false; }
                    }
                }
                None => return false,
            }
        }
        if !self.pseudo_classes.is_empty() { return false; }
        true
    }
}

impl Stylesheet {
    pub fn parse(css_text: &str) -> Stylesheet {
        parser::parse_stylesheet(css_text)
    }

    pub fn match_rules<'a>(&'a self, node: &Node) -> Vec<(&'a HashMap<String, String>, u32)> {
        let mut matched = Vec::new();
        for rule in &self.rules {
            for selector in &rule.selectors {
                if matches_complex(selector, node) {
                    let spec = compute_specificity(selector);
                    matched.push((&rule.declarations, spec));
                    break;
                }
            }
        }
        matched.sort_by_key(|&(_, spec)| spec);
        matched
    }
}

fn compute_specificity(sel: &[SelectorComponent]) -> u32 {
    let (mut id, mut cls, mut tag) = (0, 0, 0);
    for c in sel {
        if let SelectorComponent::Compound(cs) = c {
            let (i, c, t) = cs.specificity();
            id += i; cls += c; tag += t;
        }
    }
    (id << 20) | (cls << 10) | tag
}

fn matches_complex(sel: &[SelectorComponent], node: &Node) -> bool {
    if sel.is_empty() { return false; }

    let compounds: Vec<&CompoundSelector> = sel.iter().filter_map(|c| {
        if let SelectorComponent::Compound(cs) = c { Some(cs) } else { None }
    }).collect();
    let combinators: Vec<&Combinator> = sel.iter().filter_map(|c| match c {
        SelectorComponent::Combinator(comb) => Some(comb),
        _ => None,
    }).collect();

    if compounds.is_empty() { return false; }
    if compounds.len() == 1 { return compounds[0].matches(node); }

    let last = compounds[compounds.len() - 1];
    if !last.matches(node) { return false; }

    let mut current = node as *const Node;
    let mut idx = compounds.len() - 1;

    for comb in combinators.iter().rev() {
        if idx == 0 { return true; }
        idx -= 1;
        let prev = compounds[idx];

        let parent = unsafe { &*current }.get_parent();
        let parent = match parent {
            Some(p) => p,
            None => return false,
        };

        match comb {
            Combinator::Child => {
                if !prev.matches(parent) { return false; }
                current = parent as *const Node;
            }
            Combinator::Descendant => {
                let mut found = false;
                let mut anc = Some(parent);
                while let Some(a) = anc {
                    if prev.matches(a) { found = true; current = a as *const Node; break; }
                    anc = a.get_parent();
                }
                if !found { return false; }
            }
        }
    }
    true
}
