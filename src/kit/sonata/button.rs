use std::sync::Arc;

use iced::border::Radius;
use iced::widget::button::Status;
use iced::widget::text;
use iced::widget::{Button, button};
use iced::{Background, Border, Color, Padding, Shadow, Vector, widget};

use crate::definition::button::{ButtonBuilder, ButtonHierarchy, ButtonSize, UiButtonProperties};
use crate::kit::sonata::Sonata;
use crate::kit::sonata::text::text::{TextStyle, TextStyle::TextSmM};
use crate::{Kit, UiButton};

impl Sonata {
    fn style(status: Status, hier: Option<ButtonHierarchy>) -> widget::button::Style {
        let hier = hier.unwrap_or(ButtonHierarchy::Primary);
        let mut style = widget::button::Style::default();

        match (status, hier) {
            // Primary styles
            (Status::Active, ButtonHierarchy::Primary) => {
                style.background = Some(Background::Color(Color::from_rgb8(0, 108, 255)))
            }
            (Status::Hovered, ButtonHierarchy::Primary) => {
                style.background = Some(Background::Color(Color::from_rgb8(0, 97, 230)))
            }
            (Status::Pressed, ButtonHierarchy::Primary) => {
                style.background = Some(Background::Color(Color::from_rgb8(0, 97, 230)))
            }
            (Status::Disabled, ButtonHierarchy::Primary) => {
                style.background = Some(Background::Color(Color::from_rgb8(127, 181, 255)))
            }

            // Secondary styles
            (Status::Active, ButtonHierarchy::Secondary) => {
                style.background = Some(Background::Color(Color::from_rgb8(230, 231, 232)))
            }
            (Status::Hovered, ButtonHierarchy::Secondary) => {
                style.background = Some(Background::Color(Color::from_rgb8(207, 208, 209)))
            }
            (Status::Pressed, ButtonHierarchy::Secondary) => {
                style.background = Some(Background::Color(Color::from_rgb8(230, 231, 232)))
            }
            (Status::Disabled, ButtonHierarchy::Secondary) => {
                style.background = Some(Background::Color(Color::from_rgb8(242, 243, 243)))
            }
        }

        style.border = Border {
            radius: Radius::from(10),
            ..Default::default()
        };

        style.shadow = Shadow {
            color: Color::from_rgba8(50, 145, 233, 0.5),
            offset: Vector::ZERO,
            blur_radius: 10.0,
        };

        style
    }
}

impl<Message: Clone + 'static> ButtonBuilder<Message> for Sonata {
    fn apply_size(
        &self,
        button: Button<'static, Message>,
        size: ButtonSize,
    ) -> Button<'static, Message> {
        let padding = match size {
            ButtonSize::SM => Padding {
                top: 7.0,
                right: 15.0,
                bottom: 7.0,
                left: 15.0,
            },
            ButtonSize::MD => Padding {
                top: 9.0,
                right: 19.0,
                bottom: 9.0,
                left: 19.0,
            },
            ButtonSize::LG => Padding {
                top: 11.0,
                right: 19.0,
                bottom: 11.0,
                left: 19.0,
            },
        };

        button.padding(padding)
    }

    fn apply_hier(
        &self,
        button: Button<'static, Message>,
        hier: ButtonHierarchy,
    ) -> Button<'static, Message> {
        button.style(move |_, status| Sonata::style(status, Some(hier.clone())))
    }

    fn button(&self, label: String, on_press: Message) -> Button<'static, Message> {
        let el = button(
            text(label)
                .font(TextStyle::build_font(TextStyle::TextSmM))
                .size(14)
                .style(|_| text::Style {
                    color: Some(Color::from_rgb8(255, 255, 255)),
                }),
        )
        .padding(Padding {
            top: 7.0,
            right: 15.0,
            bottom: 7.0,
            left: 15.0,
        })
        .on_press(on_press)
        .style(move |_, status| Sonata::style(status, Some(ButtonHierarchy::Primary)));

        el
    }
}

impl<Message: Clone + 'static> Kit<Message> for Sonata {
    fn button<'a>(
        &'a self,
        label: impl Into<String>,
        on_press: Message,
    ) -> UiButton<Message, Self> {
        let el = button(
            text(label.into())
                .font(TextStyle::build_font(TextStyle::TextSmM))
                .size(14)
                .style(|_| text::Style {
                    color: Some(Color::from_rgb8(255, 255, 255)),
                }),
        )
        .padding(Padding {
            top: 7.0,
            right: 15.0,
            bottom: 7.0,
            left: 15.0,
        })
        .on_press(on_press)
        .style(move |_, status| Sonata::style(status, Some(ButtonHierarchy::Primary)));

        UiButton {
            inner: el,
            kit: Arc::new(self.clone()),
        }
    }
}
