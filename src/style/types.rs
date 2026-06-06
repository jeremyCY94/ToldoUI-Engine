#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxSizing { ContentBox, BorderBox }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f32), Auto, Percent(f32),
    #[allow(dead_code)] FitContent,
    #[allow(dead_code)] MaxContent,
    #[allow(dead_code)] MinContent,
}

impl Length {
    #[allow(dead_code)]
    pub fn px(v: f32) -> Self { Length::Px(v) }
    pub fn to_dim(&self) -> taffy::Dimension {
        match self { Length::Px(v) => taffy::Dimension::Length(*v), Length::Percent(v) => taffy::Dimension::Percent(*v), _ => taffy::Dimension::Auto }
    }
    pub fn to_lp(&self) -> taffy::LengthPercentage {
        match self { Length::Px(v) => taffy::LengthPercentage::Length(*v), Length::Percent(v) => taffy::LengthPercentage::Percent(*v), _ => taffy::LengthPercentage::Length(0.0) }
    }
    pub fn to_lpa(&self) -> taffy::LengthPercentageAuto {
        match self { Length::Px(v) => taffy::LengthPercentageAuto::Length(*v), Length::Percent(v) => taffy::LengthPercentageAuto::Percent(*v), _ => taffy::LengthPercentageAuto::Auto }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }
impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self { Color { r, g, b, a } }
    pub fn black() -> Self { Color::new(0,0,0,255) }
    #[allow(dead_code)]
    pub fn white() -> Self { Color::new(255,255,255,255) }
    pub fn transparent() -> Self { Color::new(0,0,0,0) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign { Left, Center, Right, Justify }
impl Default for TextAlign { fn default() -> Self { TextAlign::Left } }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position { Static, Relative, Absolute, Fixed, Sticky }
impl Default for Position { fn default() -> Self { Position::Static } }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderLineStyle { None, Solid, Dashed, Dotted, Double }

#[derive(Debug, Clone)]
pub struct BorderSide { pub width: f32, pub style: BorderLineStyle, pub color: Color }
impl BorderSide {
    pub fn none() -> Self { BorderSide { width: 0.0, style: BorderLineStyle::None, color: Color::transparent() } }
    pub fn to_border(&self) -> taffy::LengthPercentage { taffy::LengthPercentage::Length(self.width) }
}

#[derive(Debug, Clone)]
pub struct BorderStyle {
    pub top: BorderSide, pub right: BorderSide, pub bottom: BorderSide, pub left: BorderSide,
}
impl Default for BorderStyle {
    fn default() -> Self { BorderStyle { top: BorderSide::none(), right: BorderSide::none(), bottom: BorderSide::none(), left: BorderSide::none() } }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    pub angle: f32,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cursor {
    Auto,
    Default,
    Pointer,
    Text,
    Wait,
    Help,
    NotAllowed,
    Progress,
    Grab,
    Grabbing,
    Move,
    ZoomIn,
    ZoomOut,
}

impl Default for Cursor {
    fn default() -> Self { Cursor::Auto }
}

