use iced::Element;

use crate::{
    definition::content_stack::UiContentStack,
    kit::sonata::{Sonata, components::content_stack::widget::ContentStack},
};

impl<Message: Clone + 'static> Sonata<Message> {
    pub fn content_stack<'a>(&self, params: UiContentStack<'a, Message>) -> Element<'a, Message> {
        let widget = ContentStack::from_vec(params.childs, params.child_index);
        widget.into()
    }
}
