use iced::Element;

use crate::definition::{button::UiButton, input::UiInput, sidebar::UiSidebar, window::UiWindow};

pub mod appstate;
pub mod cached;
pub mod definition;
pub mod kit;
pub mod mapper;
pub mod router;

pub trait KitProvider: Clone + 'static {
    fn provide<M: Clone + Send + 'static>(&self) -> Box<dyn for<'a> Kit<'a, M>>;
}

pub type KitObj<Message> = Box<dyn for<'a> Kit<'a, Message>>;

pub trait Kit<'a, Message: Clone + 'static> {
    // Primitive
    fn constr_button(&self, btn: UiButton<'a, Message>) -> Element<'a, Message>;
    fn constr_input(&self, input: UiInput<'a, Message>) -> Element<'a, Message>;

    // Complicated
    fn constr_window(&self, window: UiWindow<'a, Message>) -> Element<'a, Message>;
    fn constr_sidebar(&self, sidebar: UiSidebar<'a, Message>) -> Element<'a, Message>;
}
