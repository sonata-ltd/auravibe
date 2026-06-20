use iced::Element;

use crate::KitObj;

pub struct UiContentStack<'a, Message> {
    pub childs: Vec<Element<'a, Message>>,
    pub child_index: usize,

    pub kit: &'a KitObj<Message>,
}

impl<'a, Message> UiContentStack<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>) -> Self {
        UiContentStack {
            childs: Vec::new(),
            child_index: 0,

            kit,
        }
    }

    pub fn with_children(
        mut self,
        current_index: usize,
        childs: Box<[Element<'a, Message>]>,
    ) -> Self {
        self.childs = Vec::from(childs);
        self.child_index = current_index;

        self
    }
}

impl<'a, Message> From<UiContentStack<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'static,
{
    fn from(value: UiContentStack<'a, Message>) -> Self {
        value.kit.constr_content_stack(value)
    }
}
