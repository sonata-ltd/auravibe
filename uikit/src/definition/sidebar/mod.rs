use iced::{Element, Length};

use crate::KitObj;

pub struct UiSidebar<'a, Message> {
    pub content: Vec<Element<'a, Message>>,
    pub width: Length,
    pub collapsed: bool,
    pub show_builtin_header_controls: bool,

    pub collpase_toggle_message: Option<Message>,

    pub kit: &'a KitObj<Message>,
}

impl<'a, Message> UiSidebar<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>, children: Vec<Element<'a, Message>>) -> Self {
        UiSidebar {
            content: children,
            width: Length::Fill,
            collapsed: false,
            show_builtin_header_controls: false,

            collpase_toggle_message: None,

            kit,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn collapse(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn show_header(mut self, show: bool) -> Self {
        self.show_builtin_header_controls = show;
        self
    }

    pub fn on_toggle(mut self, msg: Message) -> Self {
        self.collpase_toggle_message = Some(msg);
        self
    }
}

impl<'a, Message: Clone + 'static> From<UiSidebar<'a, Message>> for Element<'a, Message> {
    fn from(value: UiSidebar<'a, Message>) -> Self {
        value.kit.constr_sidebar(value)
    }
}
