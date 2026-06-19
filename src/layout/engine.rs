use std::collections::HashMap;
use std::rc::Rc;

use taffy::TaffyTree;

use crate::dom::{self, Node, NodeType};
use crate::style::{ComputedStyle, StyleMap};

pub type LayoutMap = HashMap<*const Node, taffy::Layout>;

pub struct LayoutEngine {
    pub taffy: TaffyTree,
    pub node_map: HashMap<*const Node, taffy::NodeId>,
    pub results: LayoutMap,
}

impl LayoutEngine {
    pub fn new() -> Self {
        LayoutEngine { taffy: TaffyTree::new(), node_map: HashMap::new(), results: HashMap::new() }
    }

    pub fn layout(&mut self, styles: &StyleMap, root: Rc<Node>, width: f32, height: f32) {
        self.taffy = TaffyTree::new();
        self.node_map.clear();
        self.results.clear();
        let mut stack = Vec::new();
        self.build_tree(&root, styles, width, &mut stack);

        // Root taffy node = first Element in traversal (html)
        let root_id = stack.first().and_then(|p| self.node_map.get(p).copied());

        if let Some(rid) = root_id {
            // Set root element size to viewport so block layout fills it
            if let Ok(root_ref) = self.taffy.style(rid) { let mut root_style = root_ref.clone();
                if root_style.size.width == taffy::Dimension::Auto {
                    root_style.size.width = taffy::Dimension::Length(width);
                }
                if root_style.size.height == taffy::Dimension::Auto {
                    root_style.size.height = taffy::Dimension::Length(height);
                }
                let _ = self.taffy.set_style(rid, root_style);
            }
            let _ = self.taffy.compute_layout(
                rid,
                taffy::Size {
                    width: taffy::AvailableSpace::Definite(width),
                    height: taffy::AvailableSpace::Definite(height),
                },
            );
        }

        self.collect_results(&root);
    }

    fn build_tree(&mut self, node: &Rc<Node>, styles: &StyleMap, cw: f32, stack: &mut Vec<*const Node>) {
        let ptr = dom::node_ptr(node);
        match &node.node_type {
            NodeType::Element(_) => {
                if stack.is_empty() {
                    stack.push(ptr); // first element = root candidate
                }
                let style = match styles.get(&ptr) {
                    Some(s) => s.clone(),
                    None => return,
                };
                if style.display == taffy::Display::None { return; }

                let mut child_ids = Vec::new();
                for child in &node.children {
                    let cptr = dom::node_ptr(child);
                    let skip = styles.get(&cptr).map(|s| s.display == taffy::Display::None).unwrap_or(false);
                    if !skip {
                        self.build_tree(child, styles, cw, stack);
                        if let Some(cid) = self.node_map.get(&cptr) { child_ids.push(*cid); }
                    }
                }

                let mut ts = style.to_taffy();
                let (tw, th) = tex_measure(node, &style);
                if tw > 0.0 && ts.size.width == taffy::Dimension::Auto {
                    let w = match style.box_sizing {
                        crate::style::BoxSizing::ContentBox => tw,
                        crate::style::BoxSizing::BorderBox => {
                            let pb = |l: &crate::style::Length| match l { crate::style::Length::Px(v) => *v, _ => 0.0 };
                            let hp = pb(&style.padding_left) + pb(&style.padding_right);
                            let hb = style.border.left.width + style.border.right.width;
                            tw + hp + hb
                        }
                    };
                    ts.size.width = taffy::Dimension::Length(w.min(cw));
                }
                if th > 0.0 && ts.size.height == taffy::Dimension::Auto {
                    let h = match style.box_sizing {
                        crate::style::BoxSizing::ContentBox => th,
                        crate::style::BoxSizing::BorderBox => {
                            let pb = |l: &crate::style::Length| match l { crate::style::Length::Px(v) => *v, _ => 0.0 };
                            let vp = pb(&style.padding_top) + pb(&style.padding_bottom);
                            let vb = style.border.top.width + style.border.bottom.width;
                            th + vp + vb
                        }
                    };
                    ts.size.height = taffy::Dimension::Length(h);
                }

                let tid = if child_ids.is_empty() {
                    self.taffy.new_leaf(ts).ok()
                } else {
                    self.taffy.new_with_children(ts, &child_ids).ok()
                };
                if let Some(t) = tid { self.node_map.insert(ptr, t); }
            }
            NodeType::Text(_) => {
                let parent = node.get_parent();
                let ps = parent.and_then(|p| styles.get(&dom::node_ptr_ref(p))).cloned().unwrap_or_default();
                let text = match &node.node_type { NodeType::Text(t) => t.clone(), _ => String::new() };
                if text.trim().is_empty() { return; }

                let fs = ps.font_size;
                let tw = text.len() as f32 * fs * 0.6;
                let th = fs * 1.4;

                let is_block = ps.display != taffy::Display::Flex;
                let ts = taffy::Style {
                    size: taffy::Size {
                        width: if is_block { taffy::Dimension::Auto } else { taffy::Dimension::Length(tw.min(cw * 0.9)) },
                        height: taffy::Dimension::Length(th),
                    },
                    ..Default::default()
                };
                if let Ok(t) = self.taffy.new_leaf(ts) { self.node_map.insert(ptr, t); }
            }
            NodeType::Document => {
                for child in &node.children { self.build_tree(child, styles, cw, stack); }
            }
        }
    }

    fn collect_results(&mut self, node: &Rc<Node>) {
        let ptr = dom::node_ptr(node);
        if let Some(&tid) = self.node_map.get(&ptr) {
            if let Ok(l) = self.taffy.layout(tid) { self.results.insert(ptr, *l); }
        }
        for child in &node.children { self.collect_results(child); }
    }

    pub fn get(&self, ptr: *const Node) -> Option<taffy::Layout> {
        self.results.get(&ptr).copied()
    }

    /// Total height of content after layout (y + height of deepest node)
    pub fn content_height(&self, root: &Rc<Node>) -> f32 {
        self.compute_max_bottom(root, 0.0)
    }

    fn compute_max_bottom(&self, node: &Rc<Node>, offset: f32) -> f32 {
        let ptr = dom::node_ptr(node);
        let mut max = offset;
        if let Some(l) = self.results.get(&ptr) {
            let bottom = offset + l.location.y + l.size.height;
            if bottom > max { max = bottom; }
        }
        for child in &node.children {
            let child_max = self.compute_max_bottom(child, offset);
            if child_max > max { max = child_max; }
        }
        max
    }
}

fn tex_measure(node: &Node, style: &ComputedStyle) -> (f32, f32) {
    if let Some(tag) = node.tag_name() {
        if tag == "textarea" || tag == "input" || tag == "select" || tag == "button" {
            return (0.0, 0.0);
        }
    }
    let mut w = 0.0; let mut lines = 0;
    for child in &node.children {
        if let NodeType::Text(ref t) = child.node_type {
            if !t.trim().is_empty() { w += t.len() as f32 * style.font_size * 0.6; lines += 1; }
        }
    }
    (w, if lines > 0 { lines as f32 * style.font_size * 1.4 } else { 0.0 })
}
