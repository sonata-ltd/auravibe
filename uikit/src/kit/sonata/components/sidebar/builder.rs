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
        let mut sidebar_widget = Sidebar::with_children(param.content).width(param.width);

        if param.collapsed == true {
            sidebar_widget = sidebar_widget.collapsed(true);
        }

        if param.show_builtin_header_controls == true {
            sidebar_widget = sidebar_widget.show_header();
        }

        if let Some(msg) = param.collpase_toggle_message {
            sidebar_widget = sidebar_widget.on_toggle(msg);
        }

        sidebar_widget.into()
    }
}
