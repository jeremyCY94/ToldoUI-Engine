use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::style::ComputedStyle;

#[derive(Debug, Clone)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Document,
    Element(ElementData),
    Text(String),
}

#[derive(Debug)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Rc<Node>>,
    pub parent: Cell<Option<*const Node>>,
    #[allow(dead_code)]
    pub computed_style: RefCell<Option<ComputedStyle>>,
}

impl Node {
    pub fn new_document() -> Rc<Node> {
        Rc::new(Node {
            node_type: NodeType::Document,
            children: Vec::new(),
            parent: Cell::new(None),
            computed_style: RefCell::new(None),
        })
    }

    pub fn new_element(tag_name: String, attributes: HashMap<String, String>) -> Rc<Node> {
        Rc::new(Node {
            node_type: NodeType::Element(ElementData { tag_name, attributes }),
            children: Vec::new(),
            parent: Cell::new(None),
            computed_style: RefCell::new(None),
        })
    }

    pub fn new_text(text: String) -> Rc<Node> {
        Rc::new(Node {
            node_type: NodeType::Text(text),
            children: Vec::new(),
            parent: Cell::new(None),
            computed_style: RefCell::new(None),
        })
    }

    pub fn is_element(&self) -> bool {
        matches!(self.node_type, NodeType::Element(_))
    }

    #[allow(dead_code)]
    pub fn is_text(&self) -> bool {
        matches!(self.node_type, NodeType::Text(_))
    }

    pub fn element_data(&self) -> Option<&ElementData> {
        match &self.node_type {
            NodeType::Element(e) => Some(e),
            _ => None,
        }
    }

    pub fn tag_name(&self) -> Option<&str> {
        self.element_data().map(|e| e.tag_name.as_str())
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.element_data()
            .and_then(|e| e.attributes.get(name).map(|s| s.as_str()))
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.get_attribute("class")
            .map(|c| c.split_whitespace().any(|p| p == class))
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn id(&self) -> Option<&str> {
        self.get_attribute("id")
    }

    pub fn get_parent(&self) -> Option<&Node> {
        self.parent.get().map(|p| unsafe { &*p })
    }

    pub fn set_parent(&self, p: *const Node) {
        self.parent.set(Some(p));
    }

    #[allow(dead_code)]
    pub fn children_text(&self) -> String {
        let mut text = String::new();
        for child in &self.children {
            match &child.node_type {
                NodeType::Text(t) => text.push_str(t),
                _ => text.push_str(&child.children_text()),
            }
        }
        text
    }

    pub fn add_child(parent: &Rc<Node>, child: &Rc<Node>) {
        child.set_parent(&**parent as *const Node);
        unsafe { (&mut *Rc::as_ptr(&parent).cast_mut()).children.push(child.clone()); }
    }
}

pub fn node_ptr(node: &Rc<Node>) -> *const Node {
    Rc::as_ptr(node)
}

pub fn node_ptr_ref(node: &Node) -> *const Node {
    node as *const Node
}
