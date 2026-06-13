use std::collections::HashMap;
use std::rc::Rc;
use std::sync::OnceLock;

use crate::css::Stylesheet;
use crate::dom::{self, Node, NodeType};
use super::types::{Position, Length, BorderSide, BorderStyle, BorderLineStyle, BoxSizing, Color, TextAlign, LinearGradient, GradientStop, Cursor};
use super::computed::ComputedStyle;

pub type StyleMap = HashMap<*const Node, ComputedStyle>;

static USER_AGENT_STYLESHEET: OnceLock<Stylesheet> = OnceLock::new();

fn get_user_agent_stylesheet() -> &'static Stylesheet {
    USER_AGENT_STYLESHEET.get_or_init(|| {
        let css = include_str!("user_agent.css");
        Stylesheet::parse(css)
    })
}

pub fn resolve_styles(stylesheet: &Stylesheet, root: Rc<Node>, hovered_node: Option<*const Node>, focused_node_key: Option<&str>) -> StyleMap {
    let mut map = StyleMap::new();
    resolve_node(stylesheet, &root, &ComputedStyle::default(), &mut map, hovered_node, focused_node_key);
    map
}

fn resolve_node(ss: &Stylesheet, node: &Rc<Node>, parent: &ComputedStyle, map: &mut StyleMap, hovered_node: Option<*const Node>, focused_node_key: Option<&str>) {
    let ptr = dom::node_ptr(node);
    let style = match &node.node_type {
        NodeType::Element(_) => {
            // Estilo base (sin hover ni focus)
            let mut style_base = ComputedStyle::default();
            style_base.inherit(parent);

            // 1. Estilos por defecto del User Agent
            let ua_matched_base = get_user_agent_stylesheet().match_rules(node, None, None);
            for (d, _) in ua_matched_base {
                apply_decls(&mut style_base, d);
            }

            // 2. Estilos de la hoja de estilos del usuario (Stylesheet)
            let matched_base = ss.match_rules(node, None, None);
            for (d, _) in matched_base {
                apply_decls(&mut style_base, d);
            }

            // 3. Atributos HTML (ej. width y height)
            if let Some(ed) = node.element_data() {
                apply_html_attrs(&mut style_base, &ed.attributes);
            }

            // 4. Estilos en línea (inline style - mayor prioridad)
            if let Some(inline) = node.get_attribute("style") {
                let fake = format!("dummy {{{}}}", inline);
                let sheet = Stylesheet::parse(&fake);
                if let Some(rule) = sheet.rules.first() {
                    apply_decls(&mut style_base, &rule.declarations);
                }
            }

            let is_hovered = crate::css::is_node_hovered(node, hovered_node);
            let is_focused = crate::css::is_node_focused(node, focused_node_key);

            if is_hovered || is_focused {
                let mut style_state = ComputedStyle::default();
                style_state.inherit(parent);

                // 1. Estilos por defecto del User Agent (con hover y/o focus)
                let ua_matched_state = get_user_agent_stylesheet().match_rules(node, hovered_node, focused_node_key);
                for (d, _) in ua_matched_state {
                    apply_decls(&mut style_state, d);
                }

                // 2. Estilos de la hoja de estilos del usuario (con hover y/o focus)
                let matched_state = ss.match_rules(node, hovered_node, focused_node_key);
                for (d, _) in matched_state {
                    apply_decls(&mut style_state, d);
                }

                // 3. Atributos HTML
                if let Some(ed) = node.element_data() {
                    apply_html_attrs(&mut style_state, &ed.attributes);
                }

                // 4. Estilos en línea
                if let Some(inline) = node.get_attribute("style") {
                    let fake = format!("dummy {{{}}}", inline);
                    let sheet = Stylesheet::parse(&fake);
                    if let Some(rule) = sheet.rules.first() {
                        apply_decls(&mut style_state, &rule.declarations);
                    }
                }

                // Lógica de botón por defecto si no tiene un fondo personalizado en hover
                if node.tag_name() == Some("button") && is_hovered {
                    let has_css_hover_bg = style_state.background_color != style_base.background_color
                        || style_state.background_gradient != style_base.background_gradient;
                    if !has_css_hover_bg {
                        style_state.background_color = style_base.background_color.darken(0.15);
                    }
                }

                style_state
            } else {
                style_base
            }
        }
        NodeType::Text(_) => {
            let mut s = ComputedStyle::default();
            s.inherit(parent);
            s
        }
        _ => parent.clone(),
    };

    map.insert(ptr, style);

    let parent_style = map.get(&ptr).cloned().unwrap_or_else(|| parent.clone());
    for child in &node.children {
        resolve_node(ss, child, &parent_style, map, hovered_node, focused_node_key);
    }
}

fn apply_decls(style: &mut ComputedStyle, d: &HashMap<String, String>) {
    for (p, v) in d { apply_prop(style, p, v); }
}

fn apply_prop(style: &mut ComputedStyle, prop: &str, value: &str) {
    let v = value.trim();
    if v.is_empty() { return; }
    match prop {
        "display" => { style.display = match v { "none" => taffy::Display::None, "flex" | "inline-flex" => taffy::Display::Flex, _ => taffy::Display::Block }; }
        "width" | "inline-size" => style.width = parse_len(v),
        "height" | "block-size" => style.height = parse_len(v),
        "min-width" => style.min_width = parse_len(v),
        "min-height" => style.min_height = parse_len(v),
        "max-width" => style.max_width = parse_len(v),
        "max-height" => style.max_height = parse_len(v),
        "margin" => { let m = parse_quad(v); style.margin_top=m[0]; style.margin_right=m[1]; style.margin_bottom=m[2]; style.margin_left=m[3]; }
        "margin-top" => style.margin_top = parse_len(v),
        "margin-right" => style.margin_right = parse_len(v),
        "margin-bottom" => style.margin_bottom = parse_len(v),
        "margin-left" => style.margin_left = parse_len(v),
        "padding" => { let p = parse_quad(v); style.padding_top=p[0]; style.padding_right=p[1]; style.padding_bottom=p[2]; style.padding_left=p[3]; }
        "padding-top" => style.padding_top = parse_len(v),
        "padding-right" => style.padding_right = parse_len(v),
        "padding-bottom" => style.padding_bottom = parse_len(v),
        "padding-left" => style.padding_left = parse_len(v),
        "border" => style.border = parse_border_shorthand(v),
        "border-top" => { if let Some(b) = parse_single_border(v) { style.border.top = b; } }
        "border-right" => { if let Some(b) = parse_single_border(v) { style.border.right = b; } }
        "border-bottom" => { if let Some(b) = parse_single_border(v) { style.border.bottom = b; } }
        "border-left" => { if let Some(b) = parse_single_border(v) { style.border.left = b; } }
        "box-sizing" => style.box_sizing = if v == "border-box" { BoxSizing::BorderBox } else { BoxSizing::ContentBox },
        "flex-direction" => { style.flex_direction = match v { "row-reverse" => taffy::FlexDirection::RowReverse, "column" => taffy::FlexDirection::Column, "column-reverse" => taffy::FlexDirection::ColumnReverse, _ => taffy::FlexDirection::Row }; }
        "flex-wrap" => { style.flex_wrap = match v { "wrap" => taffy::FlexWrap::Wrap, "wrap-reverse" => taffy::FlexWrap::WrapReverse, _ => taffy::FlexWrap::NoWrap }; }
        "flex-flow" => { for p in v.split_whitespace() { match p { "row"|"row-reverse"|"column"|"column-reverse" => apply_prop(style,"flex-direction",p), "nowrap"|"wrap"|"wrap-reverse" => apply_prop(style,"flex-wrap",p), _ => {} } } }
        "flex" => {
            let parts: Vec<&str> = v.split_whitespace().collect();
            if parts.len()==3 { style.flex_grow=parse_f32(parts[0]); style.flex_shrink=parse_f32(parts[1]); style.flex_basis=parse_len(parts[2]); }
            else if parts.len()==2 {
                if parts[1].contains("px")||parts[1].contains('%')||parts[1]=="auto"||parts[1]=="content" { style.flex_grow=parse_f32(parts[0]); style.flex_basis=parse_len(parts[1]); }
                else { style.flex_grow=parse_f32(parts[0]); style.flex_shrink=parse_f32(parts[1]); }
            } else if parts.len()==1 { match v { "auto" => { style.flex_grow=1.0;style.flex_shrink=1.0;style.flex_basis=Length::Auto; } "none" => { style.flex_grow=0.0;style.flex_shrink=0.0;style.flex_basis=Length::Auto; } _ => { let n=parse_f32(v);if n!=0.0||v.contains('.'){style.flex_grow=n;}else{style.flex_basis=parse_len(v);} } } }
        }
        "flex-grow" => style.flex_grow = parse_f32(v),
        "flex-shrink" => style.flex_shrink = parse_f32(v),
        "flex-basis" => style.flex_basis = parse_len(v),
        "justify-content" => { style.justify_content = match v { "flex-start"=>Some(taffy::JustifyContent::FlexStart),"flex-end"=>Some(taffy::JustifyContent::FlexEnd),"center"=>Some(taffy::JustifyContent::Center),"space-between"=>Some(taffy::JustifyContent::SpaceBetween),"space-around"=>Some(taffy::JustifyContent::SpaceAround),"space-evenly"=>Some(taffy::JustifyContent::SpaceEvenly),"stretch"=>Some(taffy::JustifyContent::Stretch),_=>style.justify_content }; }
        "align-items" => { style.align_items = match v { "flex-start"=>Some(taffy::AlignItems::FlexStart),"flex-end"=>Some(taffy::AlignItems::FlexEnd),"center"=>Some(taffy::AlignItems::Center),"baseline"=>Some(taffy::AlignItems::Baseline),"stretch"=>Some(taffy::AlignItems::Stretch),_=>style.align_items }; }
        "align-self" => { style.align_self = match v { "auto"=>None,"flex-start"=>Some(taffy::AlignSelf::FlexStart),"flex-end"=>Some(taffy::AlignSelf::FlexEnd),"center"=>Some(taffy::AlignSelf::Center),"baseline"=>Some(taffy::AlignSelf::Baseline),"stretch"=>Some(taffy::AlignSelf::Stretch),_=>style.align_self }; }
        "align-content" => { style.align_content = match v { "flex-start"=>Some(taffy::AlignContent::FlexStart),"flex-end"=>Some(taffy::AlignContent::FlexEnd),"center"=>Some(taffy::AlignContent::Center),"space-between"=>Some(taffy::AlignContent::SpaceBetween),"space-around"=>Some(taffy::AlignContent::SpaceAround),"space-evenly"=>Some(taffy::AlignContent::SpaceEvenly),"stretch"=>Some(taffy::AlignContent::Stretch),_=>style.align_content }; }
        "gap"|"grid-gap" => { let p:Vec<&str>=v.split_whitespace().collect(); if p.len()>=2{style.gap_row=parse_len(p[0]);style.gap_column=parse_len(p[1]);}else if p.len()==1{let l=parse_len(p[0]);style.gap_row=l;style.gap_column=l;} }
        "row-gap" => style.gap_row = parse_len(v),
        "column-gap" => style.gap_column = parse_len(v),
        "overflow" => { let p:Vec<&str>=v.split_whitespace().collect(); let o=parse_overflow(p.first().copied().unwrap_or("visible")); style.overflow_x=o; style.overflow_y=p.get(1).map(|&s|parse_overflow(s)).unwrap_or(o); }
        "overflow-x" => style.overflow_x = parse_overflow(v),
        "overflow-y" => style.overflow_y = parse_overflow(v),
        "background" => {
            if v.starts_with("linear-gradient") {
                if let Some(grad) = parse_linear_gradient(v) {
                    style.background_gradient = Some(grad);
                }
            } else if let Some(c) = parse_color(v) {
                style.background_color = c;
                style.background_gradient = None;
            }
        }
        "background-image" => {
            if let Some(grad) = parse_linear_gradient(v) {
                style.background_gradient = Some(grad);
            }
        }
        "background-color" => {
            if let Some(c) = parse_color(v) {
                style.background_color = c;
            }
        }
        "color" => { if let Some(c)=parse_color(v){style.color=c;} }
        "font-size" => style.font_size = parse_font_size(v, style.font_size),
        "font-family" => style.font_family = v.split(',').next().unwrap_or("sans-serif").trim().trim_matches('"').trim_matches('\'').to_string(),
        "font-weight" => { style.font_weight = match v { "normal"=>400,"bold"=>700,"lighter"=>300,"bolder"=>900, v=>{let n=parse_f32(v)as u16;if n>=100&&n<=900{n}else{400}} }; }
        "text-align" => { style.text_align = match v { "left"=>TextAlign::Left,"center"=>TextAlign::Center,"right"=>TextAlign::Right,"justify"=>TextAlign::Justify,_=>TextAlign::Left }; }
        "line-height" => { style.line_height = if v=="normal"{Length::Px(style.font_size*1.2)}else{parse_len(v)}; }
        "opacity" => style.opacity = parse_f32(v).max(0.0).min(1.0),
        "visibility" => style.visibility = v != "hidden" && v != "collapse",
        "cursor" => {
            style.cursor = match v.to_lowercase().as_str() {
                "default" => Cursor::Default,
                "pointer" => Cursor::Pointer,
                "text" => Cursor::Text,
                "wait" => Cursor::Wait,
                "help" => Cursor::Help,
                "not-allowed" => Cursor::NotAllowed,
                "progress" => Cursor::Progress,
                "grab" => Cursor::Grab,
                "grabbing" => Cursor::Grabbing,
                "move" => Cursor::Move,
                "zoom-in" => Cursor::ZoomIn,
                "zoom-out" => Cursor::ZoomOut,
                _ => Cursor::Auto,
            };
        }
        "position" => { style.position = match v { "relative"=>Position::Relative,"absolute"=>Position::Absolute,"fixed"=>Position::Fixed,"sticky"=>Position::Sticky,_=>Position::Static }; }
        "top" => style.top = parse_len(v), "right" => style.right = parse_len(v),
        "bottom" => style.bottom = parse_len(v), "left" => style.left = parse_len(v),
        "border-radius" => style.border_radius = parse_len(v),
        _ => {}
    }
}

fn apply_html_attrs(s: &mut ComputedStyle, attrs: &HashMap<String, String>) {
    if let Some(w) = attrs.get("width").and_then(|w| w.parse::<f32>().ok()) {
        if s.width == Length::Auto { s.width = Length::Px(w); }
    }
    if let Some(h) = attrs.get("height").and_then(|h| h.parse::<f32>().ok()) {
        if s.height == Length::Auto { s.height = Length::Px(h); }
    }
}

fn parse_len(v: &str) -> Length {
    let v = v.trim();
    if v=="auto"||v.is_empty(){return Length::Auto;}
    if v.ends_with("px"){if let Ok(n)=v.trim_end_matches("px").trim().parse::<f32>(){return Length::Px(n);}}
    else if v.ends_with('%'){if let Ok(n)=v.trim_end_matches('%').trim().parse::<f32>(){return Length::Percent(n/100.0);}}
    else if v.ends_with("em"){if let Ok(n)=v.trim_end_matches("em").trim().parse::<f32>(){return Length::Px(n*16.0);}}
    else if v.ends_with("rem"){if let Ok(n)=v.trim_end_matches("rem").trim().parse::<f32>(){return Length::Px(n*16.0);}}
    else if v.ends_with("pt"){if let Ok(n)=v.trim_end_matches("pt").trim().parse::<f32>(){return Length::Px(n*1.33333);}}
    else if let Ok(n)=v.parse::<f32>(){return Length::Px(n);}
    Length::Auto
}

fn parse_quad(v: &str) -> [Length;4] {
    let p:Vec<&str>=v.split_whitespace().collect();
    match p.len(){1=>{let l=parse_len(p[0]);[l,l,l,l]}2=>{let a=parse_len(p[0]);let b=parse_len(p[1]);[a,b,a,b]}3=>{let a=parse_len(p[0]);let b=parse_len(p[1]);let c=parse_len(p[2]);[a,b,c,b]}4=>{[parse_len(p[0]),parse_len(p[1]),parse_len(p[2]),parse_len(p[3])]}_=>[Length::Px(0.0);4]}
}

fn parse_f32(v: &str)->f32{v.trim().parse().unwrap_or(0.0)}

fn parse_color(v: &str)->Option<Color>{
    let v=v.trim();
    if v=="transparent"{return Some(Color::transparent());}
    if let Some(h)=v.strip_prefix('#'){return parse_hex(h);}
    if let Some(n)=named_color(v){return Some(n);}
    if v.starts_with("rgb"){return parse_rgb(v);}
    None
}

fn parse_hex(h:&str)->Option<Color>{
    match h.len(){
        3=>Some(Color::new(u8::from_str_radix(&h[0..1],16).ok()?*17,u8::from_str_radix(&h[1..2],16).ok()?*17,u8::from_str_radix(&h[2..3],16).ok()?*17,255)),
        6=>Some(Color::new(u8::from_str_radix(&h[0..2],16).ok()?,u8::from_str_radix(&h[2..4],16).ok()?,u8::from_str_radix(&h[4..6],16).ok()?,255)),
        8=>{let a=u8::from_str_radix(&h[6..8],16).ok()?;Some(Color::new(u8::from_str_radix(&h[0..2],16).ok()?,u8::from_str_radix(&h[2..4],16).ok()?,u8::from_str_radix(&h[4..6],16).ok()?,a))}
        _=>None}
}

fn parse_rgb(v:&str)->Option<Color>{
    let v = v.trim();
    let inner = match v.strip_prefix("rgba(").or_else(|| v.strip_prefix("rgb(")) {
        Some(s) => s.strip_suffix(')').unwrap_or("").trim(),
        None => return None,
    };
    let parts:Vec<&str>=inner.split(|c|c==','||c==' '||c=='/').map(|s|s.trim()).filter(|s|!s.is_empty()).collect();
    if parts.len()<3{return None;}
    let r=if parts[0].ends_with('%'){(parts[0].trim_end_matches('%').parse::<f32>().ok()?*255.0/100.0)as u8}else{parts[0].parse::<u8>().ok()?};
    let g=if parts[1].ends_with('%'){(parts[1].trim_end_matches('%').parse::<f32>().ok()?*255.0/100.0)as u8}else{parts[1].parse::<u8>().ok()?};
    let b=if parts[2].ends_with('%'){(parts[2].trim_end_matches('%').parse::<f32>().ok()?*255.0/100.0)as u8}else{parts[2].parse::<u8>().ok()?};
    let a=if parts.len()>3{(parse_f32(parts[3])*255.0).round().max(0.0).min(255.0)as u8}else{255};
    Some(Color::new(r,g,b,a))
}

fn named_color(n:&str)->Option<Color>{
    Some(match n.to_lowercase().as_str(){
        "black"=>Color::new(0,0,0,255),"silver"=>Color::new(192,192,192,255),"gray"|"grey"=>Color::new(128,128,128,255),
        "white"=>Color::new(255,255,255,255),"maroon"=>Color::new(128,0,0,255),"red"=>Color::new(255,0,0,255),
        "purple"=>Color::new(128,0,128,255),"fuchsia"|"magenta"=>Color::new(255,0,255,255),
        "green"=>Color::new(0,128,0,255),"lime"=>Color::new(0,255,0,255),"olive"=>Color::new(128,128,0,255),
        "yellow"=>Color::new(255,255,0,255),"navy"=>Color::new(0,0,128,255),"blue"=>Color::new(0,0,255,255),
        "teal"=>Color::new(0,128,128,255),"aqua"|"cyan"=>Color::new(0,255,255,255),
        "aliceblue"=>Color::new(240,248,255,255),"antiquewhite"=>Color::new(250,235,215,255),
        "aquamarine"=>Color::new(127,255,212,255),"azure"=>Color::new(240,255,255,255),
        "beige"=>Color::new(245,245,220,255),"bisque"=>Color::new(255,228,196,255),
        "blanchedalmond"=>Color::new(255,235,205,255),"blueviolet"=>Color::new(138,43,226,255),
        "brown"=>Color::new(165,42,42,255),"burlywood"=>Color::new(222,184,135,255),
        "cadetblue"=>Color::new(95,158,160,255),"chartreuse"=>Color::new(127,255,0,255),
        "chocolate"=>Color::new(210,105,30,255),"coral"=>Color::new(255,127,80,255),
        "cornflowerblue"=>Color::new(100,149,237,255),"cornsilk"=>Color::new(255,248,220,255),
        "crimson"=>Color::new(220,20,60,255),"darkblue"=>Color::new(0,0,139,255),
        "darkcyan"=>Color::new(0,139,139,255),"darkgoldenrod"=>Color::new(184,134,11,255),
        "darkgray"|"darkgrey"=>Color::new(169,169,169,255),"darkgreen"=>Color::new(0,100,0,255),
        "darkkhaki"=>Color::new(189,183,107,255),"darkmagenta"=>Color::new(139,0,139,255),
        "darkolivegreen"=>Color::new(85,107,47,255),"darkorange"=>Color::new(255,140,0,255),
        "darkorchid"=>Color::new(153,50,204,255),"darkred"=>Color::new(139,0,0,255),
        "darksalmon"=>Color::new(233,150,122,255),"darkseagreen"=>Color::new(143,188,143,255),
        "darkslateblue"=>Color::new(72,61,139,255),"darkslategray"|"darkslategrey"=>Color::new(47,79,79,255),
        "darkturquoise"=>Color::new(0,206,209,255),"darkviolet"=>Color::new(148,0,211,255),
        "deeppink"=>Color::new(255,20,147,255),"deepskyblue"=>Color::new(0,191,255,255),
        "dimgray"|"dimgrey"=>Color::new(105,105,105,255),"dodgerblue"=>Color::new(30,144,255,255),
        "firebrick"=>Color::new(178,34,34,255),"floralwhite"=>Color::new(255,250,240,255),
        "forestgreen"=>Color::new(34,139,34,255),"gainsboro"=>Color::new(220,220,220,255),
        "ghostwhite"=>Color::new(248,248,255,255),"gold"=>Color::new(255,215,0,255),
        "goldenrod"=>Color::new(218,165,32,255),"greenyellow"=>Color::new(173,255,47,255),
        "honeydew"=>Color::new(240,255,240,255),"hotpink"=>Color::new(255,105,180,255),
        "indianred"=>Color::new(205,92,92,255),"indigo"=>Color::new(75,0,130,255),
        "ivory"=>Color::new(255,255,240,255),"khaki"=>Color::new(240,230,140,255),
        "lavender"=>Color::new(230,230,250,255),"lavenderblush"=>Color::new(255,240,245,255),
        "lawngreen"=>Color::new(124,252,0,255),"lemonchiffon"=>Color::new(255,250,205,255),
        "lightblue"=>Color::new(173,216,230,255),"lightcoral"=>Color::new(240,128,128,255),
        "lightcyan"=>Color::new(224,255,255,255),"lightgoldenrodyellow"=>Color::new(250,250,210,255),
        "lightgray"|"lightgrey"=>Color::new(211,211,211,255),"lightgreen"=>Color::new(144,238,144,255),
        "lightpink"=>Color::new(255,182,193,255),"lightsalmon"=>Color::new(255,160,122,255),
        "lightseagreen"=>Color::new(32,178,170,255),"lightskyblue"=>Color::new(135,206,250,255),
        "lightslategray"|"lightslategrey"=>Color::new(119,136,153,255),"lightsteelblue"=>Color::new(176,196,222,255),
        "lightyellow"=>Color::new(255,255,224,255),"limegreen"=>Color::new(50,205,50,255),
        "linen"=>Color::new(250,240,230,255),"mediumaquamarine"=>Color::new(102,205,170,255),
        "mediumblue"=>Color::new(0,0,205,255),"mediumorchid"=>Color::new(186,85,211,255),
        "mediumpurple"=>Color::new(147,112,219,255),"mediumseagreen"=>Color::new(60,179,113,255),
        "mediumslate_blue"=>Color::new(123,104,238,255),"mediumspringgreen"=>Color::new(0,250,154,255),
        "mediumturquoise"=>Color::new(72,209,204,255),"mediumvioletred"=>Color::new(199,21,133,255),
        "midnightblue"=>Color::new(25,25,112,255),"mintcream"=>Color::new(245,255,250,255),
        "mistyrose"=>Color::new(255,228,225,255),"moccasin"=>Color::new(255,228,181,255),
        "navajowhite"=>Color::new(255,222,173,255),"oldlace"=>Color::new(253,245,230,255),
        "olivedrab"=>Color::new(107,142,35,255),"orange"=>Color::new(255,165,0,255),
        "orangered"=>Color::new(255,69,0,255),"orchid"=>Color::new(218,112,214,255),
        "palegoldenrod"=>Color::new(238,232,170,255),"palegreen"=>Color::new(152,251,152,255),
        "paleturquoise"=>Color::new(175,238,238,255),"palevioletred"=>Color::new(219,112,147,255),
        "papayawhip"=>Color::new(255,239,213,255),"peachpuff"=>Color::new(255,218,185,255),
        "peru"=>Color::new(205,133,63,255),"pink"=>Color::new(255,192,203,255),
        "plum"=>Color::new(221,160,221,255),"powderblue"=>Color::new(176,224,230,255),
        "rebeccapurple"=>Color::new(102,51,153,255),"rosybrown"=>Color::new(188,143,143,255),
        "royalblue"=>Color::new(65,105,225,255),"saddlebrown"=>Color::new(139,69,19,255),
        "salmon"=>Color::new(250,128,114,255),"sandybrown"=>Color::new(244,164,96,255),
        "seagreen"=>Color::new(46,139,87,255),"seashell"=>Color::new(255,245,238,255),
        "sienna"=>Color::new(160,82,45,255),"skyblue"=>Color::new(135,206,235,255),
        "slateblue"=>Color::new(106,90,205,255),"slategray"|"slategrey"=>Color::new(112,128,144,255),
        "snow"=>Color::new(255,250,250,255),"springgreen"=>Color::new(0,255,127,255),
        "steelblue"=>Color::new(70,130,180,255),"tan"=>Color::new(210,180,140,255),
        "thistle"=>Color::new(216,191,216,255),"tomato"=>Color::new(255,99,71,255),
        "turquoise"=>Color::new(64,224,208,255),"violet"=>Color::new(238,130,238,255),
        "wheat"=>Color::new(245,222,179,255),"whitesmoke"=>Color::new(245,245,245,255),
        "yellowgreen"=>Color::new(154,205,50,255),
        _=>return None,
    })
}

fn parse_font_size(v:&str,parent:f32)->f32{
    let v=v.trim();
    match v{
        "small"=>13.0,"medium"=>16.0,"large"=>18.0,"x-small"=>10.0,"xx-small"=>8.0,"x-large"=>24.0,"xx-large"=>32.0,
        "smaller"=>parent*0.833,"larger"=>parent*1.2,
        v=>{if v.ends_with("em"){parse_f32(v.trim_end_matches("em").trim())*parent}else if v.ends_with("rem"){parse_f32(v.trim_end_matches("rem").trim())*16.0}else if v.ends_with('%'){parse_f32(v.trim_end_matches('%').trim())*parent/100.0}else if v.ends_with("px"){parse_f32(v.trim_end_matches("px").trim())}else if v.ends_with("pt"){parse_f32(v.trim_end_matches("pt").trim())*1.33333}else if let Ok(n)=v.parse::<f32>(){if n<=100.0{parent*n/100.0}else{n}}else{parent}}
    }
}

fn parse_overflow(v:&str)->taffy::Overflow{
    match v{"visible"=>taffy::Overflow::Visible,"hidden"|"clip"=>taffy::Overflow::Hidden,"scroll"=>taffy::Overflow::Scroll,_=>taffy::Overflow::Visible}
}

fn parse_border_width(v:&str)->f32{
    let v=v.trim().trim_end_matches("px").trim();
    v.parse::<f32>().unwrap_or(0.0)
}
fn split_css_values(v: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    for c in v.chars() {
        if c == '(' {
            depth += 1;
            current.push(c);
        } else if c == ')' {
            if depth > 0 { depth -= 1; }
            current.push(c);
        } else if c.is_whitespace() && depth == 0 {
            if !current.trim().is_empty() {
                parts.push(current.trim().to_string());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn parse_border_shorthand(v:&str)->BorderStyle{
    let parts = split_css_values(v);
    let mut w=0.0;let mut c=Color::black();let mut s=BorderLineStyle::None;
    for p in &parts{if let Some(cl)=parse_color(p){c=cl;}else{match p.as_str(){"none"=>s=BorderLineStyle::None,"solid"=>s=BorderLineStyle::Solid,"dashed"=>s=BorderLineStyle::Dashed,"dotted"=>s=BorderLineStyle::Dotted,"double"=>s=BorderLineStyle::Double,_=>{let n=parse_border_width(p);if n>0.0||p=="0"{w=n;}else{w=1.0;}}}}}
    BorderStyle{top:BorderSide{width:w,style:s,color:c},right:BorderSide{width:w,style:s,color:c},bottom:BorderSide{width:w,style:s,color:c},left:BorderSide{width:w,style:s,color:c}}
}

fn parse_single_border(v:&str)->Option<BorderSide>{
    let parts = split_css_values(v);
    let mut w=0.0;let mut c=Color::black();let mut s=BorderLineStyle::None;
    for p in &parts{if let Some(cl)=parse_color(p){c=cl;}else{match p.as_str(){"none"=>s=BorderLineStyle::None,"solid"=>s=BorderLineStyle::Solid,"dashed"=>s=BorderLineStyle::Dashed,"dotted"=>s=BorderLineStyle::Dotted,"double"=>s=BorderLineStyle::Double,_=>{w=parse_border_width(p);}}}}
    Some(BorderSide{width:w,style:s,color:c})
}

pub fn parse_linear_gradient(v: &str) -> Option<LinearGradient> {
    let v = v.trim();
    if !v.starts_with("linear-gradient(") || !v.ends_with(')') {
        return None;
    }
    let content = v.strip_prefix("linear-gradient(").unwrap().strip_suffix(')').unwrap().trim();
    let parts = split_commas_top_level(content);
    if parts.is_empty() { return None; }

    let mut start_idx = 0;
    let mut angle = 180.0; // por defecto 'to bottom'

    if let Some(parsed_angle) = parse_angle(&parts[0]) {
        angle = parsed_angle;
        start_idx = 1;
    }

    let mut raw_stops = Vec::new();
    for part in &parts[start_idx..] {
        if let Some(stop) = parse_color_stop(part) {
            raw_stops.push(stop);
        } else {
            return None;
        }
    }

    if raw_stops.len() < 2 {
        return None;
    }

    let stops = distribute_stops(raw_stops);
    Some(LinearGradient { angle, stops })
}

fn split_commas_top_level(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    for c in s.chars() {
        if c == '(' {
            depth += 1;
            current.push(c);
        } else if c == ')' {
            if depth > 0 {
                depth -= 1;
            }
            current.push(c);
        } else if c == ',' && depth == 0 {
            parts.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(c);
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn parse_angle(s: &str) -> Option<f32> {
    let s = s.trim().to_lowercase();
    if s.ends_with("deg") {
        if let Ok(val) = s.trim_end_matches("deg").trim().parse::<f32>() {
            return Some(val);
        }
    }
    if s.starts_with("to ") {
        let dir = s.strip_prefix("to ").unwrap().trim();
        return Some(match dir {
            "top" => 0.0,
            "right" => 90.0,
            "bottom" => 180.0,
            "left" => 270.0,
            "top right" | "right top" => 45.0,
            "bottom right" | "right bottom" => 135.0,
            "bottom left" | "left bottom" => 225.0,
            "top left" | "left top" => 315.0,
            _ => 180.0,
        });
    }
    None
}

fn parse_color_stop(s: &str) -> Option<(Color, Option<f32>)> {
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() { return None; }
    let last = parts.last().unwrap();
    if last.ends_with('%') {
        if let Ok(pct) = last.trim_end_matches('%').parse::<f32>() {
            let color_str = s.trim_end_matches(last).trim();
            if let Some(color) = parse_color(color_str) {
                return Some((color, Some(pct / 100.0)));
            }
        }
    }
    if let Some(color) = parse_color(s) {
        return Some((color, None));
    }
    None
}

fn distribute_stops(mut stops: Vec<(Color, Option<f32>)>) -> Vec<GradientStop> {
    if stops.is_empty() { return Vec::new(); }
    if stops.len() == 1 {
        return vec![GradientStop {
            position: 0.0,
            color: stops[0].0,
        }];
    }

    if stops[0].1.is_none() {
        stops[0].1 = Some(0.0);
    }
    let last_idx = stops.len() - 1;
    if stops[last_idx].1.is_none() {
        stops[last_idx].1 = Some(1.0);
    }

    let mut i = 0;
    while i < stops.len() {
        if stops[i].1.is_none() {
            let mut j = i + 1;
            while j < stops.len() && stops[j].1.is_none() {
                j += 1;
            }
            let start_val = stops[i - 1].1.unwrap();
            let end_val = stops[j].1.unwrap();
            let count = (j - i + 1) as f32;
            let step = (end_val - start_val) / count;
            for k in i..j {
                stops[k].1 = Some(start_val + step * (k - i + 1) as f32);
            }
            i = j;
        } else {
            i += 1;
        }
    }

    stops.into_iter().map(|(color, pos)| GradientStop {
        position: pos.unwrap_or(0.0),
        color,
    }).collect()
}
