use std::sync::LazyLock;

use iced::Color;

pub mod button;
pub mod input;
pub mod sidebar;
pub mod window;

pub static COMPONENT_DEBUG_COLOR: LazyLock<Color> =
    LazyLock::new(|| Color::from_rgb8(134, 12, 190));

// #[macro_export]
// macro_rules! ui {
//     ($kit:expr $expr:expr) => {
//         $expr.constr()
//     };
// }
