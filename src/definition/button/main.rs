use iced::{Element, Length};

use crate::{
    Kit,
    definition::button::props::{ButtonHierarchy, ButtonSize, UiButtonProperties},
};

pub struct UiButton<'a, Message> {
    pub label: String,
    pub on_press: Message,
    pub props: UiButtonProperties,
    pub width: Length,

    pub kit: &'a Box<dyn Kit<Message>>,
}

impl<'a, Message> UiButton<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(
        kit: &'a Box<dyn Kit<Message>>,
        label: impl Into<String>,
        on_press: Message,
    ) -> Self {
        UiButton {
            label: label.into(),
            on_press,
            props: UiButtonProperties::default(),
            width: Length::Shrink,
            kit,
        }
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.props.set_size(size);
        self
    }

    pub fn hier(mut self, hier: ButtonHierarchy) -> Self {
        self.props.set_hier(hier);
        self
    }
}

impl<'a, Message: Clone + 'static> From<UiButton<'a, Message>> for Element<'static, Message> {
    fn from(value: UiButton<Message>) -> Self {
        value.kit.constr_button(value)
    }
}
