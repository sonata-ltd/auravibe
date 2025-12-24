use iced::Element;

use crate::definition::button::main::UiButton;

pub mod definition;
pub mod kit;
pub mod mapper;

#[derive(Debug, Clone)]
pub enum KitMessage {
    ButtonPressed,
}

pub trait Kit<Message: Clone + 'static> {
    fn constr_button(&self, btn: UiButton<Message>) -> Element<'static, Message>;
}
