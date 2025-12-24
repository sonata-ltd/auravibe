use crate::definition::button::{ButtonBuilder, UiButton};

pub mod definition;
pub mod helper;
pub mod kit;

#[derive(Debug, Clone)]
pub enum KitMessage {
    ButtonPressed,
}

pub trait Kit<Message>: ButtonBuilder<Message> {
    fn button(&self, label: impl Into<String>, on_press: Message) -> UiButton<Message, Self>
    where
        Self: Sized;
}
