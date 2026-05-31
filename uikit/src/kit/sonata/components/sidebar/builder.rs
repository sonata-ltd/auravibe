use iced::Element;

use crate::{
    definition::sidebar::UiSidebar,
    kit::sonata::{Sonata, components::sidebar::widget::Sidebar},
};

impl<'a, Message> Sonata<Message>
where
    Message: Clone + 'static,
{
    pub fn sidebar<'b>(&self, param: UiSidebar<'b, Message>) -> Element<'b, Message> {
        Sidebar::with_children(param.content)
            .width(param.width)
            .into()
    }
}
