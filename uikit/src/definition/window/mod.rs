use std::f32;

use iced::{Element, TextureCache};

use crate::KitObj;

pub struct UiWindow<'a, Message> {
    pub label: String,
    // pub child: Element<'a, Message>,
    pub controls_child: Option<Element<'a, Message>>,

    pub wrapper: bool,
    pub wrapper_close_event: Option<Message>,
    pub content_stack_childs: (usize, Vec<Element<'a, Message>>),
    pub max_height: Option<f32>,
    pub width: Option<f32>,

    pub header_texture_cache: TextureCache,

    pub kit: &'a KitObj<Message>,
}

impl<'a, Message> UiWindow<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>, label: String) -> Self {
        UiWindow {
            label,
            // child: Vec::new(),
            controls_child: None,

            wrapper: false,
            wrapper_close_event: None,
            content_stack_childs: (0, Vec::new()),
            max_height: None,
            width: None,

            header_texture_cache: TextureCache::new(),

            kit,
        }
    }

    pub fn with_children(
        mut self,
        current_index: usize,
        childs: Box<[Element<'a, Message>]>,
    ) -> Self {
        self.content_stack_childs = (current_index, Vec::from(childs));
        self
    }

    pub fn controls_child(mut self, child: impl Into<Element<'a, Message>>) -> Self {
        self.controls_child = Some(child.into());
        self
    }

    pub fn wrap(mut self, enable: bool) -> Self {
        self.wrapper = enable;
        self
    }

    pub fn close_event(mut self, event: Message) -> Self {
        self.wrapper = true;
        self.wrapper_close_event = Some(event);
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl<'a, Message: Clone + 'static> From<UiWindow<'a, Message>> for Element<'a, Message> {
    fn from(value: UiWindow<'a, Message>) -> Self {
        value.kit.constr_window(value)
    }
}
