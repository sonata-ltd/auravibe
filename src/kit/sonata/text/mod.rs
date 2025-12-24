use iced::{
    Font,
    font::{Family, Stretch, Style, Weight},
};

pub mod display;
pub mod text;

pub struct FontToken {
    pub weight: Weight,
    pub style: Style,
}

impl<'a> display::DisplayStyle {
    const fn token(self) -> &'a FontToken {
        &display::TOKENS[self as usize]
    }

    pub fn build_font(self) -> Font {
        let token = Self::token(self);

        Font {
            family: Family::Name("Hack"),
            weight: token.weight,
            stretch: Stretch::Normal,
            style: token.style,
        }
    }
}

impl<'a> text::TextStyle {
    const fn token(self) -> &'a FontToken {
        &text::TOKENS[self as usize]
    }

    pub fn build_font(self) -> Font {
        let token = Self::token(self);

        Font {
            family: Family::Name("Inter"),
            weight: token.weight,
            stretch: Stretch::Normal,
            style: token.style,
        }
    }
}
