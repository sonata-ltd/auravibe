use iced::{Element, widget::button};

use crate::{helper::UIMapper, kit::sonata::ComponentId};

impl<'a, AppMsg: 'static> UIMapper<'a, AppMsg> {
    pub fn button(&mut self, label: impl Into<String>, event: AppMsg) -> Element<'a, AppMsg> {
        let mapper = self.get_mapper();

        self.kit
            .button(label.into(), event)
            .map(move |kmsg| (mapper)(kmsg))
    }
}
