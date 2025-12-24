use iced::font::{Style, Weight};

use crate::kit::sonata::text::FontToken;

/// Typography variants for the application's UI kit (Text styles).
/// Naming follows: Text <Subtype> <Line Height/Font Size>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextStyle {
    // TEXT XL – 20/30
    /// Text XL Regular - 20/30
    TextXlR,
    /// Text XL Medium - 20/30
    TextXlM,
    /// Text XL Semibold - 20/30
    TextXlSm,
    /// Text XL Bold - 20/30
    TextXlB,
    /// Text XL Regular Italic - 20/30
    TextXlRi,
    /// Text XL Medium Italic - 20/30
    TextXlMi,
    /// Text XL Semibold Italic - 20/30
    TextXlSmi,
    /// Text XL Bold Italic - 20/30
    TextXlBi,
    /// Text XL Regular Underlined - 20/30
    TextXlRu,
    /// Text XL Medium Underlined - 20/30
    TextXlMu,

    // TEXT LG – 18/28
    /// Text LG Regular - 18/28
    TextLgR,
    /// Text LG Medium - 18/28
    TextLgM,
    /// Text LG Semibold - 18/28
    TextLgSm,
    /// Text LG Bold - 18/28
    TextLgB,
    /// Text LG Regular Italic - 18/28
    TextLgRi,
    /// Text LG Medium Italic - 18/28
    TextLgMi,
    /// Text LG Semibold Italic - 18/28
    TextLgSmi,
    /// Text LG Bold Italic - 18/28
    TextLgBi,
    /// Text LG Regular Underlined - 18/28
    TextLgRu,
    /// Text LG Medium Underlined - 18/28
    TextLgMu,

    // TEXT MD – 16/24
    /// Text MD Regular - 16/24
    TextMdR,
    /// Text MD Medium - 16/24
    TextMdM,
    /// Text MD Semibold - 16/24
    TextMdSm,
    /// Text MD Bold - 16/24
    TextMdB,
    /// Text MD Regular Italic - 16/24
    TextMdRi,
    /// Text MD Medium Italic - 16/24
    TextMdMi,
    /// Text MD Semibold Italic - 16/24
    TextMdSmi,
    /// Text MD Bold Italic - 16/24
    TextMdBi,
    /// Text MD Regular Underlined - 16/24
    TextMdRu,

    // TEXT SM – 14/20
    /// Text SM Regular - 14/20
    TextSmR,
    /// Text SM Medium - 14/20
    TextSmM,
    /// Text SM Semibold - 14/20
    TextSmSm,
    /// Text SM Bold - 14/20
    TextSmB,

    // TEXT XS – 12/18
    /// Text XS Regular - 12/18
    TextXsR,
    /// Text XS Medium - 12/18
    TextXsM,
    /// Text XS Semibold - 12/18
    TextXsSm,
    /// Text XS Bold - 12/18
    TextXsB,
}

/// Static font table indexed by Text as usize.
/// Order must match exactly the order of enum variants.
pub const TOKENS: &[FontToken] = &[
    // TEXT XL
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    }, // R
    FontToken {
        weight: Weight::Medium,
        style: Style::Normal,
    }, // M
    FontToken {
        weight: Weight::Semibold,
        style: Style::Normal,
    }, // SM
    FontToken {
        weight: Weight::Bold,
        style: Style::Normal,
    }, // B
    FontToken {
        weight: Weight::Normal,
        style: Style::Italic,
    }, // RI
    FontToken {
        weight: Weight::Medium,
        style: Style::Italic,
    }, // MI
    FontToken {
        weight: Weight::Semibold,
        style: Style::Italic,
    }, // SMI
    FontToken {
        weight: Weight::Bold,
        style: Style::Italic,
    }, // BI
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    }, // RU (underlined handled outside)
    FontToken {
        weight: Weight::Medium,
        style: Style::Normal,
    }, // MU
    // TEXT LG
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
        weight: Weight::Normal,
        style: Style::Italic,
    },
    FontToken {
        weight: Weight::Medium,
        style: Style::Italic,
    },
    FontToken {
        weight: Weight::Semibold,
        style: Style::Italic,
    },
    FontToken {
        weight: Weight::Bold,
        style: Style::Italic,
    },
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    }, // RU
    FontToken {
        weight: Weight::Medium,
        style: Style::Normal,
    }, // MU
    // TEXT MD
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
        weight: Weight::Normal,
        style: Style::Italic,
    },
    FontToken {
        weight: Weight::Medium,
        style: Style::Italic,
    },
    FontToken {
        weight: Weight::Semibold,
        style: Style::Italic,
    },
    FontToken {
        weight: Weight::Bold,
        style: Style::Italic,
    },
    FontToken {
        weight: Weight::Normal,
        style: Style::Normal,
    }, // RU
    // TEXT SM
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
    // TEXT XS
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
