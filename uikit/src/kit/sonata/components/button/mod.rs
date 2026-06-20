use iced::Alignment::Center;
use iced::Length::Fill;
use iced::widget::button::Status;
use iced::widget::{Space, text};
use iced::widget::{Text, button, row};
use iced::{Border, Element, Padding, Renderer, Shadow, Theme, widget};

use crate::UiButton;
use crate::definition::button::Content;
use crate::definition::button::props::{ButtonHierarchy, ButtonSize};
use crate::kit::sonata::Sonata;
use crate::kit::sonata::components::button::vars::{
    ButtonHierarchyStyle, FOCUS_BORDER_OFFSET, FOCUS_BORDER_RADIUS, FOCUS_BORDER_WIDTH,
    PADDING_BUTTON_LG, PADDING_BUTTON_MD, PADDING_BUTTON_SM, RADIUS_BUTTON, SHADOW_PRIMARY,
    STYLE_DESTRUCTIVE, STYLE_PRIMARY, STYLE_SECONDARY, STYLE_TERTIARY,
};
use crate::kit::sonata::utils::text::text::TextStyle;
use crate::kit::sonata::utils::widgets::multi_border::{Appearance, BorderLayer, multiborder};

mod vars;

fn style_for(hier: &ButtonHierarchy) -> &'static ButtonHierarchyStyle {
    match hier {
        ButtonHierarchy::Primary => &STYLE_PRIMARY,
        ButtonHierarchy::Secondary => &STYLE_SECONDARY,
        ButtonHierarchy::Tertiary => &STYLE_TERTIARY,
        ButtonHierarchy::Destructive => &STYLE_DESTRUCTIVE,
    }
}

fn padding_for(size: &ButtonSize, content_type: &Content) -> Padding {
    let padding_hier = match size {
        ButtonSize::SM => PADDING_BUTTON_SM,
        ButtonSize::MD => PADDING_BUTTON_MD,
        ButtonSize::LG => PADDING_BUTTON_LG,
    };

    match content_type {
        Content::Text(_) => padding_hier.default,
        Content::Icon(_) => padding_hier.icon_only,
        Content::Combined { .. } => padding_hier.combined,
        Content::None => padding_hier.default,
    }
}

fn style(status: Status, hier_style: &ButtonHierarchyStyle) -> widget::button::Style {
    let colors = match status {
        Status::Active => &hier_style.active,
        Status::Hovered => &hier_style.hover,
        Status::Pressed => &hier_style.pressed(),
        Status::Disabled => &hier_style.disabled,
    };

    let shadow = if hier_style.has_shadow && status != Status::Disabled {
        SHADOW_PRIMARY
    } else {
        Shadow::default()
    };

    button::Style {
        background: Some(colors.background),
        text_color: colors.text,
        border: Border {
            radius: RADIUS_BUTTON.into(),
            ..Default::default()
        },
        shadow,
        ..Default::default()
    }
}

impl<Message: Clone + 'static> Sonata<Message> {
    pub fn button<'a>(&self, params: UiButton<'a, Message>) -> Element<'a, Message> {
        let UiButton {
            content,
            on_press,
            on_press_maybe,
            props,
            width,
            ..
        } = params;

        let size = props.get_size();
        let hier = props.get_hier();
        let hier_style = style_for(&hier);
        let is_disabled = on_press.is_none();

        let padding = padding_for(&size, &content);

        let content: Element<'a, Message> = match content {
            Content::Text(t) => text(t)
                .font(TextStyle::build_font(TextStyle::TextSmM))
                .align_x(Center)
                .width(Fill)
                .size(14)
                .into(),
            Content::Icon(i) => i.height(20.0).width(20.0).into(),
            Content::Combined { icon, text } => {
                let text = self.build_text(text);

                row![icon.height(20).width(20), text].spacing(5).into()
            }
            Content::None => Space::new().into(),
        };

        let mut btn = button(content)
            .padding(padding)
            .width(width)
            .style(move |_, status| style(status, &hier_style));

        if let Some(event) = on_press {
            btn = btn.on_press(event);
        }

        if let Some(event_maybe) = on_press_maybe {
            btn = btn.on_press_maybe(event_maybe);
        }

        let focus_color = hier_style.focus_border_color;

        multiborder(btn)
            .style(move |_, status| {
                if status.is_pressed && !is_disabled {
                    Appearance::new().layer(
                        BorderLayer::outer(FOCUS_BORDER_WIDTH, focus_color)
                            .offset(FOCUS_BORDER_OFFSET)
                            .radius(FOCUS_BORDER_RADIUS),
                    )
                } else {
                    Appearance::new()
                }
            })
            .into()
    }

    fn build_text<'a>(&self, t: &'a str) -> Text<'a, Theme, Renderer> {
        text(t)
            .font(TextStyle::build_font(TextStyle::TextSmM))
            .align_x(Center)
            .width(Fill)
            .size(14)
            .into()
    }
}
