use std::rc::Rc;

use crate::dom::Node;
use crate::dom::parser::HtmlParser;

#[derive(Debug)]
pub struct DomTree {
    pub document: Rc<Node>,
}

impl DomTree {
    #[allow(dead_code)]
    pub fn new() -> Self {
        DomTree { document: Node::new_document() }
    }

    pub fn parse_html(html: &str) -> Self {
        let doc = Node::new_document();
        let mut parser = HtmlParser::new(html);
        parser.parse_children(&doc, true);
        DomTree { document: doc }
    }

    pub fn document_element(&self) -> Option<Rc<Node>> {
        self.document.children.iter()
            .find(|c| c.is_element())
            .cloned()
    }

    #[allow(dead_code)]
    pub fn root(&self) -> Rc<Node> {
        self.document.clone()
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> DomIterator {
        DomIterator::new(self.document.clone())
    }
}

#[allow(dead_code)]
pub struct DomIterator {
    stack: Vec<Rc<Node>>,
}

#[allow(dead_code)]
impl DomIterator {
    pub fn new(root: Rc<Node>) -> Self {
        DomIterator { stack: vec![root] }
    }
}

impl Iterator for DomIterator {
    type Item = Rc<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        for child in node.children.iter().rev() {
            self.stack.push(child.clone());
        }
        Some(node)
    }
}
