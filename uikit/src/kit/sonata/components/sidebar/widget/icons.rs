use std::sync::LazyLock;

use iced::widget::svg;

static CHEVRON_LEFT: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("assets/chevron-left.svg")));
static CHEVRON_RIGHT: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("assets/chevron-right.svg")));

pub(crate) fn chevron(collapsed: bool) -> svg::Handle {
    if collapsed {
        CHEVRON_RIGHT.clone()
    } else {
        CHEVRON_LEFT.clone()
    }
}
