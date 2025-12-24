use iced::font::{Style, Weight};

use crate::kit::sonata::text::FontToken;

/// Typography variants for the application's UI kit.
/// Naming follows: Display <Subtype> <Line Height/Font Size>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayStyle {
    // Display 2XL
    /// Display 2XL Regular - 72/90
    Display2xlR,
    /// Display 2XL Medium - 72/90
    Display2xlM,
    /// Display 2XL Semibold - 72/90
    Display2xlSm,
    /// Display 2XL Bold - 72/90
    Display2xlB,

    // Display XL
    /// Display XL Regular - 60/72
    DisplayXlR,
    /// Display XL Medium - 60/72
    DisplayXlM,
    /// Display XL Semibold - 60/72
    DisplayXlSm,
    /// Display XL Bold - 60/72
    DisplayXlB,

    // Display LG
    /// Display LG Regular - 48/60
    DisplayLgR,
    /// Display LG Medium - 48/60
    DisplayLgM,
    /// Display LG Semibold - 48/60
    DisplayLgSm,
    /// Display LG Bold - 48/60
    DisplayLgB,

    // Display MD
    /// Display MD Regular - 36/44
    DisplayMdR,
    /// Display MD Medium - 36/44
    DisplayMdM,
    /// Display MD Semibold - 36/44
    DisplayMdSm,
    /// Display MD Bold - 36/44
    DisplayMdB,

    // Display SM
    /// Display SM Regular - 30/38
    DisplaySmR,
    /// Display SM Medium - 30/38
    DisplaySmM,
    /// Display SM Semibold - 30/38
    DisplaySmSm,
    /// Display SM Bold - 30/38
    DisplaySmB,
    /// Display SM Medium Italic - 30/44
    DisplaySmMi,

    // Display XS
    /// Display XS Regular - 24/32
    DisplayXsR,
    /// Display XS Medium - 24/32
    DisplayXsM,
    /// Display XS Semibold - 24/32
    DisplayXsSm,
    /// Display XS Bold - 24/32
    DisplayXsB,
}

/// Static font table indexed by Display as usize.
/// Order matches order of enum variants.
pub const TOKENS: &[FontToken] = &[
    // 2XL
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Medium,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Semibold,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Bold,
        style: Style::Normal,
    },
    // XL
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Medium,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Semibold,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Bold,
        style: Style::Normal,
    },
    // LG
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Medium,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Semibold,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Bold,
        style: Style::Normal,
    },
    // MD
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Medium,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Semibold,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Bold,
        style: Style::Normal,
    },
    // SM
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Medium,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Semibold,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Bold,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Medium,
        style: Style::Italic,
    },
    // XS
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Medium,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Semibold,
        style: Style::Normal,
    },
    FontToken {
        weight: Weight::Bold,
        style: Style::Normal,
    },
];
