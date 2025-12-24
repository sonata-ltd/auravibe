use iced::Alignment::Center;
use iced::Length::Fill;
use iced::border::Radius;
use iced::widget::button;
use iced::widget::button::Status;
use iced::widget::text;
use iced::{Background, Border, Color, Element, Padding, Shadow, Vector, widget};

use crate::definition::button::props::{ButtonHierarchy, ButtonSize};
use crate::kit::sonata::Sonata;
use crate::kit::sonata::text::text::TextStyle;
use crate::{Kit, UiButton};

impl Sonata {
    fn define_padding(size: &ButtonSize) -> Padding {
        match size {
            ButtonSize::SM => Padding {
                top: 7.0,
                right: 15.0,
                bottom: 7.0,
                left: 15.0,
            },
            ButtonSize::MD => Padding {
                top: 9.0,
                right: 17.0,
                bottom: 9.0,
                left: 17.0,
            },
            ButtonSize::LG => Padding {
                top: 11.0,
                right: 19.0,
                bottom: 11.0,
                left: 19.0,
            },
        }
    }

    fn style(status: Status, hier: Option<&ButtonHierarchy>) -> widget::button::Style {
        let hier = hier.unwrap_or(&ButtonHierarchy::Primary);
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

impl<Message: Clone + 'static> Kit<Message> for Sonata {
    fn constr_button(&self, params: UiButton<Message>) -> Element<'static, Message> {
        let UiButton {
            label,
            on_press,
            props,
            width,
            ..
        } = params;

        button(
            text(label)
                .font(TextStyle::build_font(TextStyle::TextSmM))
                .align_x(Center)
                .width(Fill)
                .size(14)
                .style(|_| text::Style {
                    color: Some(Color::from_rgb8(255, 255, 255)),
                }),
        )
        .padding(Self::define_padding(props.get_size()))
        .width(width)
        .on_press(on_press)
        .style(move |_, status| Self::style(status, Some(props.get_hier())))
        .into()
    }
}
