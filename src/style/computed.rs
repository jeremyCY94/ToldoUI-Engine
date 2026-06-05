use super::types::{Position, Length, BorderStyle, BoxSizing, Color, TextAlign};

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub display: taffy::Display, pub position: Position,
    pub width: Length, pub height: Length,
    pub min_width: Length, pub min_height: Length, pub max_width: Length, pub max_height: Length,
    pub margin_top: Length, pub margin_right: Length, pub margin_bottom: Length, pub margin_left: Length,
    pub padding_top: Length, pub padding_right: Length, pub padding_bottom: Length, pub padding_left: Length,
    pub border: BorderStyle, pub box_sizing: BoxSizing,
    pub flex_direction: taffy::FlexDirection, pub flex_wrap: taffy::FlexWrap,
    pub flex_grow: f32, pub flex_shrink: f32, pub flex_basis: Length,
    pub justify_content: Option<taffy::JustifyContent>,
    pub align_items: Option<taffy::AlignItems>,
    pub align_self: Option<taffy::AlignSelf>,
    pub align_content: Option<taffy::AlignContent>,
    pub gap_row: Length, pub gap_column: Length,
    pub overflow_x: taffy::Overflow, pub overflow_y: taffy::Overflow,
    pub background_color: Color, pub color: Color,
    pub font_size: f32, pub font_family: String, pub font_weight: u16,
    pub text_align: TextAlign, pub line_height: Length,
    pub top: Length, pub right: Length, pub bottom: Length, pub left: Length,
    pub opacity: f32, pub visibility: bool,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        ComputedStyle {
            display: taffy::Display::Block, position: Position::Static,
            width: Length::Auto, height: Length::Auto,
            min_width: Length::Px(0.0), min_height: Length::Px(0.0), max_width: Length::Auto, max_height: Length::Auto,
            margin_top: Length::Px(0.0), margin_right: Length::Px(0.0), margin_bottom: Length::Px(0.0), margin_left: Length::Px(0.0),
            padding_top: Length::Px(0.0), padding_right: Length::Px(0.0), padding_bottom: Length::Px(0.0), padding_left: Length::Px(0.0),
            border: BorderStyle::default(), box_sizing: BoxSizing::ContentBox,
            flex_direction: taffy::FlexDirection::Row, flex_wrap: taffy::FlexWrap::NoWrap,
            flex_grow: 0.0, flex_shrink: 1.0, flex_basis: Length::Auto,
            justify_content: None, align_items: None, align_self: None, align_content: None,
            gap_row: Length::Px(0.0), gap_column: Length::Px(0.0),
            overflow_x: taffy::Overflow::Visible, overflow_y: taffy::Overflow::Visible,
            background_color: Color::transparent(), color: Color::black(),
            font_size: 16.0, font_family: "sans-serif".to_string(), font_weight: 400,
            text_align: TextAlign::Center, line_height: Length::Px(1.2),
            top: Length::Auto, right: Length::Auto, bottom: Length::Auto, left: Length::Auto,
            opacity: 1.0, visibility: true,
        }
    }
}

impl ComputedStyle {
    pub fn inherit(&mut self, parent: &ComputedStyle) {
        self.color = parent.color; self.font_size = parent.font_size;
        self.font_family.clone_from(&parent.font_family);
        self.font_weight = parent.font_weight; self.text_align = parent.text_align;
        self.line_height = parent.line_height; self.visibility = parent.visibility;
    }

    pub fn to_taffy(&self) -> taffy::Style {
        let fb = match &self.flex_basis {
            Length::Auto if self.width != Length::Auto => self.width.to_dim(),
            _ => self.flex_basis.to_dim(),
        };
        taffy::Style {
            display: match self.display { taffy::Display::None => taffy::Display::None, taffy::Display::Flex => taffy::Display::Flex, _ => taffy::Display::Block },
            overflow: taffy::Point { x: self.overflow_x, y: self.overflow_y },
            position: match self.position { Position::Absolute | Position::Fixed => taffy::Position::Absolute, _ => taffy::Position::Relative },
            inset: taffy::Rect { top: self.top.to_lpa(), right: self.right.to_lpa(), bottom: self.bottom.to_lpa(), left: self.left.to_lpa() },
            size: taffy::Size { width: self.width.to_dim(), height: self.height.to_dim() },
            min_size: taffy::Size { width: self.min_width.to_dim(), height: self.min_height.to_dim() },
            max_size: taffy::Size { width: self.max_width.to_dim(), height: self.max_height.to_dim() },
            aspect_ratio: None,
            margin: taffy::Rect { top: self.margin_top.to_lpa(), right: self.margin_right.to_lpa(), bottom: self.margin_bottom.to_lpa(), left: self.margin_left.to_lpa() },
            padding: taffy::Rect { top: self.padding_top.to_lp(), right: self.padding_right.to_lp(), bottom: self.padding_bottom.to_lp(), left: self.padding_left.to_lp() },
            border: taffy::Rect { top: self.border.top.to_border(), right: self.border.right.to_border(), bottom: self.border.bottom.to_border(), left: self.border.left.to_border() },
            box_sizing: match self.box_sizing { BoxSizing::ContentBox => taffy::BoxSizing::ContentBox, BoxSizing::BorderBox => taffy::BoxSizing::BorderBox },
            flex_direction: self.flex_direction, flex_wrap: self.flex_wrap,
            flex_grow: self.flex_grow, flex_shrink: self.flex_shrink, flex_basis: fb,
            align_self: self.align_self, align_items: self.align_items,
            align_content: self.align_content, justify_content: self.justify_content,
            gap: taffy::Size { width: self.gap_column.to_lp(), height: self.gap_row.to_lp() },
            ..Default::default()
        }
    }
}
