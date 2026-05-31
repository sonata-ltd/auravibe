use iced::{Element, Length, Theme, advanced::svg, widget::Svg};

use crate::{
    KitObj,
    definition::button::props::{ButtonHierarchy, ButtonSize, UiButtonProperties},
};

pub mod props;

pub struct UiButton<'a, Message> {
    pub content: Content<'a>,
    pub props: UiButtonProperties,
    pub width: Length,

    pub on_press: Option<Message>,
    pub on_press_maybe: Option<Option<Message>>,
    pub kit: &'a KitObj<Message>,
}

type Label<'a> = &'a str;
type SvgSource<'a> = Svg<'a, Theme>;

pub enum Content<'a> {
    Text(Label<'a>),
    Icon(SvgSource<'a>),
    Combined {
        icon: SvgSource<'a>,
        text: Label<'a>,
    },
}

impl<'a> Content<'a> {
    pub fn add_text(mut self, text: Label<'a>) -> Self {
        match self {
            Content::Icon(i) => self = Content::Combined { icon: i, text },
            Content::Combined { icon, .. } => self = Content::Combined { icon, text },
            _ => self = Content::Text(text),
        };

        self
    }

    pub fn add_icon(mut self, icon: SvgSource<'a>) -> Self {
        match self {
            Content::Text(text) => self = Content::Combined { icon, text },
            Content::Combined { text, .. } => self = Content::Combined { icon, text },
            _ => self = Content::Icon(icon),
        }

        self
    }
}

impl<'a, Message> UiButton<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a KitObj<Message>) -> Self {
        UiButton {
            content: Content::Text(""),
            on_press: None,
            on_press_maybe: None,
            props: UiButtonProperties::default(),
            width: Length::Shrink,
            kit,
        }
    }

    pub fn on_press(mut self, event: Message) -> Self {
        self.on_press = Some(event);
        self
    }

    pub fn on_press_maybe(mut self, option_event: Option<Message>) -> Self {
        self.on_press_maybe = Some(option_event);
        self
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

    pub fn label(mut self, text: Label<'a>) -> Self {
        self.content = self.content.add_text(text);
        self
    }

    pub fn icon(mut self, handle: impl Into<svg::Handle>) -> Self {
        let icon = Svg::new(handle);
        self.content = self.content.add_icon(icon);
        self
    }
}

impl<'a, Message: Clone + 'static> From<UiButton<'a, Message>> for Element<'a, Message> {
    fn from(value: UiButton<'a, Message>) -> Self {
        value.kit.constr_button(value)
    }
}
