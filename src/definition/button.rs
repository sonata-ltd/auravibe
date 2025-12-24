use std::sync::Arc;

use iced::{
    Color, Element, Length,
    widget::{self, Button, Text, button},
};

use crate::Kit;

pub struct UiButton<Message, K>
where
    K: ButtonBuilder<Message>,
{
    pub(crate) inner: Button<'static, Message>,
    pub(crate) label: String,
    // pub(crate) text_color: Option<Arc<dyn Fn(&str) -> TextColor + Send + Sync>>,
    pub(crate) kit: Arc<K>,
}

pub struct TextColor {
    pub(crate) color: Color,
}

pub struct UiButtonProperties {
    pub(crate) hierarchy: ButtonHierarchy,
    pub(crate) size: ButtonSize,
}

impl Default for UiButtonProperties {
    fn default() -> Self {
        Self {
            hierarchy: ButtonHierarchy::Primary,
            size: ButtonSize::MD,
        }
    }
}

#[derive(Clone)]
pub enum ButtonHierarchy {
    Primary,
    Secondary,
    // Tertiary,
}

#[derive(Clone)]
pub enum ButtonSize {
    SM,
    MD,
    LG,
}

pub trait ButtonBuilder<Message> {
    fn apply_size(
        &self,
        button: Button<'static, Message>,
        size: ButtonSize,
    ) -> Button<'static, Message>;

    fn apply_hier(
        &self,
        button: Button<'static, Message>,
        hier: ButtonHierarchy,
    ) -> Button<'static, Message>;

    fn button(&self, label: String, on_press: Message) -> Button<'static, Message>;
}

impl<Message, K> UiButton<Message, K>
where
    K: ButtonBuilder<Message>,
{
    pub fn width(mut self, width: Length) -> Self {
        self.inner = self.inner.width(width);
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.inner = self.kit.apply_size(self.inner, size);
        self
    }

    pub fn hier(mut self, hier: ButtonHierarchy) -> Self {
        self.inner = self.kit.apply_hier(self.inner, hier);
        self
    }

    // pub fn text_style<F>(mut self, f: F) -> Self
    // where
    //     F: Fn(&str) -> TextColor + Send + Sync + 'static,
    // {
    //     self.text_style = Some(Arc::new(f));
    //     self
    // }
}

impl<Message: Clone + 'static, K> From<UiButton<Message, K>> for Element<'static, Message>
where
    K: ButtonBuilder<Message>,
{
    fn from(btn: UiButton<Message, K>) -> Self {
        println!("from");

        btn.inner.into()
    }
}
