use std::marker::PhantomData;

use iced::Element;

use crate::{
    KitObj,
    definition::{
        button::UiButton, content_stack::UiContentStack, input::UiInput, sidebar::UiSidebar,
        window::UiWindow,
    },
};

pub struct UIMapper<'a, Message> {
    kit: &'a KitObj<Message>,
    _marker: PhantomData<Message>,
}

impl<'a, Message> UIMapper<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>) -> Self {
        Self {
            kit,
            _marker: PhantomData,
        }
    }

    pub fn button(&self) -> UiButton<'a, Message> {
        UiButton::new(self.kit)
    }

    pub fn input(&self, placeholder: &'a str, value: &'a str) -> UiInput<'a, Message> {
        UiInput::new(self.kit, placeholder, value)
    }

    pub fn sidebar(&self, child: Vec<Element<'a, Message>>) -> UiSidebar<'a, Message> {
        UiSidebar::new(self.kit, child)
    }

    pub fn window(&self, label: &'a str) -> UiWindow<'a, Message> {
        UiWindow::new(self.kit, label.to_string())
    }

    pub fn content_stack(&self) -> UiContentStack<'a, Message> {
        UiContentStack::new(self.kit)
    }
}
