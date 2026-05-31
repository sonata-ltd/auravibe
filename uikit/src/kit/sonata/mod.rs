use std::marker::PhantomData;

use iced::Element;

use crate::definition::button::UiButton;
use crate::{Kit, KitProvider};

pub mod components;
pub mod palette;
pub mod utils;

#[derive(Clone)]
pub struct SonataProvider;

impl KitProvider for SonataProvider {
    fn provide<M: Clone + 'static>(&self) -> Box<dyn for<'a> Kit<'a, M>> {
        Box::new(Sonata::new())
    }
}

#[derive(Clone)]
pub struct Sonata<Message> {
    _marker: PhantomData<Message>,
}

impl<Message: Clone + 'static> Sonata<Message> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Message: Clone + 'static> Default for Sonata<Message> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<'a, Message: Clone + 'static> Kit<'a, Message> for Sonata<Message> {
    fn constr_button(&self, params: UiButton<'a, Message>) -> Element<'a, Message> {
        Self::button(&self, params)
    }

    fn constr_input(
        &self,
        input: crate::definition::input::UiInput<'a, Message>,
    ) -> Element<'a, Message> {
        Self::input(&self, input)
    }

    fn constr_sidebar(
        &self,
        sidebar: crate::definition::sidebar::UiSidebar<'a, Message>,
    ) -> Element<'a, Message> {
        Self::sidebar(&self, sidebar)
    }

    fn constr_window(
        &self,
        params: crate::definition::window::UiWindow<'a, Message>,
    ) -> Element<'a, Message> {
        Self::window(&self, params)
    }

    // fn constr_window(
    //     &self,
    //     window: crate::definition::window::UiWindow<'a, Message>,
    // ) -> Element<'a, Message> {
    //     self.window(window)
    // }
}
