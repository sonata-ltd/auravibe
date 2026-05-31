use iced::{Background, Color};

#[derive(PartialEq, Clone, Copy)]
pub enum BorderSide {
    Outer,
    Inner,
}

#[derive(Clone, Copy)]
pub struct BorderLayer {
    pub side: BorderSide,
    pub width: f32,
    pub color: Color,
    pub radius: f32,
    pub offset: f32,
}

impl BorderLayer {
    pub fn outer(width: f32, color: Color) -> Self {
        Self {
            side: BorderSide::Outer,
            width,
            color,
            radius: 0.0,
            offset: 0.0,
        }
    }

    pub fn inner(width: f32, color: Color) -> Self {
        Self {
            side: BorderSide::Inner,
            width,
            color,
            radius: 0.0,
            offset: 0.0,
        }
    }

    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }

    pub fn offset(mut self, o: f32) -> Self {
        self.offset = o;
        self
    }
}

#[derive(Debug)]
pub struct Status {
    pub is_pressed: bool,
    pub is_hovered: bool,
    pub is_focused: bool,
    pub is_disabled: bool,
}

#[derive(Default)]
pub struct Appearance {
    pub layers: Vec<BorderLayer>,
    pub background: Option<Background>,
}

impl Appearance {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn layer(mut self, layer: BorderLayer) -> Self {
        self.layers.push(layer);
        self
    }

    pub fn background(mut self, bg: impl Into<Background>) -> Self {
        self.background = Some(bg.into());
        self
    }
}
