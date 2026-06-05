use std::collections::HashMap;
use std::rc::Rc;

use crate::dom::Node;

const SELF_CLOSING: &[&str] = &["br", "hr", "img", "input", "meta", "link", "area",
    "base", "col", "embed", "source", "track", "wbr"];

pub struct HtmlParser {
    pub chars: Vec<char>,
    pub pos: usize,
}

impl HtmlParser {
    pub fn new(html: &str) -> Self {
        HtmlParser {
            chars: html.chars().collect(),
            pos: 0,
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() { self.pos += 1; } else { break; }
        }
    }

    fn expect_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    pub fn parse_children(&mut self, parent: &Rc<Node>, until_closing: bool) {
        loop {
            self.skip_whitespace();

            if self.eof() { break; }

            if self.peek() == Some('<') {
                if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '/' {
                    if until_closing { break; }
                    self.skip_to_char('>');
                    self.pos += 1;
                    continue;
                }

                if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '!' {
                    if self.pos + 2 < self.chars.len() && self.chars[self.pos + 2] == '-' 
                       && self.pos + 3 < self.chars.len() && self.chars[self.pos + 3] == '-' {
                        while self.pos + 3 < self.chars.len() 
                            && !(self.chars[self.pos] == '-' && self.chars[self.pos+1] == '-' && self.chars[self.pos+2] == '>') {
                            self.pos += 1;
                        }
                        self.pos += 3;
                    } else {
                        self.skip_to_char('>');
                        self.pos += 1;
                    }
                    continue;
                }

                if let Some(child) = self.parse_element() {
                    Node::add_child(parent, &child);
                }
            } else {
                let text = self.parse_text();
                if !text.trim().is_empty() {
                    let text_node = Node::new_text(text);
                    Node::add_child(parent, &text_node);
                }
            }
        }
    }

    fn parse_element(&mut self) -> Option<Rc<Node>> {
        if !self.expect_char('<') { return None; }

        self.skip_whitespace();
        let tag_name = self.parse_identifier();
        if tag_name.is_empty() { return None; }

        let tag_lower = tag_name.to_lowercase();
        let mut attrs = HashMap::new();

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('>') => { self.pos += 1; break; }
                Some('/') => {
                    self.pos += 1;
                    if self.expect_char('>') { break; }
                    continue;
                }
                None => return None,
                _ => {
                    if let Some((key, value)) = self.parse_attribute() {
                        attrs.insert(key, value);
                    } else {
                        break;
                    }
                }
            }
        }

        let element = Node::new_element(tag_lower.clone(), attrs);

        let is_self_closing = SELF_CLOSING.contains(&tag_lower.as_str());

        if !is_self_closing {
            self.parse_children(&element, true);
            self.skip_whitespace();
            if self.peek() == Some('<') && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '/' {
                self.pos += 2;
                let _end_tag = self.parse_identifier().to_lowercase();
                self.skip_whitespace();
                self.expect_char('>');
            }
        }

        Some(element)
    }

    fn parse_text(&mut self) -> String {
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c == '<' { break; }
            text.push(c);
            self.pos += 1;
        }
        text
    }

    fn parse_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ':' { 
                ident.push(c); self.pos += 1; 
            } else { break; }
        }
        ident
    }

    fn parse_attribute(&mut self) -> Option<(String, String)> {
        self.skip_whitespace();
        let name = self.parse_identifier();
        if name.is_empty() { return None; }

        self.skip_whitespace();
        let value = if self.expect_char('=') {
            self.skip_whitespace();
            self.parse_attribute_value()
        } else {
            String::new()
        };

        Some((name.to_lowercase(), value))
    }

    fn parse_attribute_value(&mut self) -> String {
        let mut value = String::new();
        let quote = self.peek();

        if quote == Some('"') || quote == Some('\'') {
            self.pos += 1;
            let q = quote.unwrap();
            while let Some(c) = self.peek() {
                if c == q { self.pos += 1; break; }
                value.push(c);
                self.pos += 1;
            }
        } else {
            while let Some(c) = self.peek() {
                if c.is_whitespace() || c == '>' || c == '/' { break; }
                value.push(c);
                self.pos += 1;
            }
        }

        value
    }

    fn skip_to_char(&mut self, target: char) {
        while let Some(c) = self.peek() {
            if c == target { break; }
            self.pos += 1;
        }
    }
}
