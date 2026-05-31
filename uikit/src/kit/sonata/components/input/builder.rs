use iced::widget::{text, text_input};
use iced::{Background, Border, Color, Element, Renderer, Theme};

use crate::kit::sonata::Sonata;
use crate::kit::sonata::components::input::tooltip::Tooltip;
use crate::kit::sonata::components::input::vars::{
    COLOR_INPUT_BACKGROUND, COLOR_INPUT_BACKGROUND_DISABLED, COLOR_INPUT_BORDER,
    COLOR_INPUT_BORDER_ERROR, COLOR_INPUT_BORDER_FOCUSED, COLOR_INPUT_BORDER_OUTER_ERROR,
    COLOR_INPUT_HINT_TEXT, COLOR_INPUT_HINT_TEXT_ERROR, COLOR_INPUT_LABEL_TEXT,
    COLOR_INPUT_PLACEHOLDER, COLOR_INPUT_TEXT, FONT_INPUT_HINT_TEXTSTYLE,
    FONT_INPUT_LABEL_TEXTSTYLE, FONT_INPUT_TEXTSTYLE, PADDING_INPUT_HORIZONTAL,
    RADIUS_INPUT_BORDER, SIZE_INPUT_HINT_TEXTSTYLE, SIZE_INPUT_LABEL_TEXTSTYLE,
    SIZE_INPUT_TEXTSTYLE, SIZE_OUTER_BORDER, SPACING_INPUT_COLUMN, WIDTH_OUTER_OFFSET_MULTIBORDER,
    WIDTH_OUTER_RADIUS_MULTIBORDER,
};
use crate::kit::sonata::palette::COLOR_FOCUS;
use crate::kit::sonata::utils::widgets::multi_border::{self, BorderLayer, multiborder};

fn border_for_status(status: text_input::Status, is_error: bool) -> (Color, f32) {
    match (status, is_error) {
        (text_input::Status::Disabled, _) => (COLOR_INPUT_BORDER, 1.0),
        (text_input::Status::Focused { .. }, false) => (Color::TRANSPARENT, 0.0),
        (text_input::Status::Focused { .. }, true) => (COLOR_INPUT_BORDER_ERROR, 1.0),
        (_, true) => (COLOR_INPUT_BORDER_ERROR, 1.0),
        (_, false) => (COLOR_INPUT_BORDER, 1.0),
    }
}

fn style(status: text_input::Status, is_error: bool) -> text_input::Style {
    let background_color = match status {
        text_input::Status::Disabled => COLOR_INPUT_BACKGROUND_DISABLED,
        _ => COLOR_INPUT_BACKGROUND,
    };

    let (border_color, border_width) = border_for_status(status, is_error);

    text_input::Style {
        background: Background::Color(background_color),
        border: Border {
            color: border_color,
            width: border_width,
            radius: RADIUS_INPUT_BORDER,
        },
        icon: Color::BLACK,
        placeholder: COLOR_INPUT_PLACEHOLDER,
        value: COLOR_INPUT_TEXT,
        selection: COLOR_FOCUS,
    }
}

impl<'a, Message> Sonata<Message>
where
    Message: Clone + 'static,
{
    pub fn input<'b>(
        &self,
        param: crate::definition::input::UiInput<'b, Message>,
    ) -> iced::Element<'b, Message> {
        let is_enabled = param.on_input.is_some();
        let is_error = param.is_error.unwrap_or(false);

        let text_input =
            self.bulid_text_input(param.placeholder, param.value, is_error, param.on_input);

        let bordered = self.wrap_multiborder(text_input.into(), is_error, is_enabled);
        let with_tooltip = self.wrap_tooltip(bordered, param.error_msg, is_error);
        let wrapped = self.wrap_label_hint(with_tooltip, param.label, param.hint, is_error);

        wrapped.into()
    }

    fn bulid_text_input<'b>(
        &self,
        placeholder: &'b str,
        value: &'b str,
        is_error: bool,
        on_input: Option<Box<dyn Fn(String) -> Message + 'static>>,
    ) -> text_input::TextInput<'b, Message, Theme, Renderer> {
        let mut el = text_input(placeholder, value)
            .font(FONT_INPUT_TEXTSTYLE)
            .size(SIZE_INPUT_TEXTSTYLE)
            .padding(PADDING_INPUT_HORIZONTAL)
            .style(move |_: &Theme, status| style(status, is_error));

        if let Some(handler) = on_input {
            el = el.on_input(handler);
        }

        el
    }

    fn wrap_multiborder<'b>(
        &self,
        input_el: Element<'b, Message, Theme, Renderer>,
        is_error: bool,
        is_enabled: bool,
    ) -> impl Into<Element<'b, Message>> {
        multiborder(input_el)
            .disabled(!is_enabled)
            .style(move |_theme, status| {
                if status.is_disabled {
                    return multi_border::Appearance::new();
                }

                let border_color = match (status.is_focused, is_error) {
                    (true, true) => COLOR_INPUT_BORDER_OUTER_ERROR,
                    (true, false) => COLOR_INPUT_BORDER_FOCUSED,
                    (false, _) => Color::TRANSPARENT,
                };

                multi_border::Appearance::new().layer(
                    BorderLayer::outer(WIDTH_OUTER_RADIUS_MULTIBORDER, border_color)
                        .radius(SIZE_OUTER_BORDER)
                        .offset(WIDTH_OUTER_OFFSET_MULTIBORDER),
                )
            })
    }

    fn wrap_label_hint<'b>(
        &self,
        input_el: impl Into<Element<'b, Message>>,
        label: Option<&'b str>,
        hint: Option<&'b str>,
        is_error: bool,
    ) -> Element<'b, Message> {
        if label.is_none() && hint.is_none() {
            return input_el.into();
        }

        let mut column = iced::widget::Column::new();

        if let Some(label) = label {
            column = column.push(
                text(label)
                    .font(FONT_INPUT_LABEL_TEXTSTYLE)
                    .size(SIZE_INPUT_LABEL_TEXTSTYLE)
                    .color(COLOR_INPUT_LABEL_TEXT),
            );
        }

        column = column.push(input_el);

        if let Some(hint_value) = hint {
            let color = if is_error {
                COLOR_INPUT_HINT_TEXT_ERROR
            } else {
                COLOR_INPUT_HINT_TEXT
            };

            column = column.push(
                text(hint_value)
                    .font(FONT_INPUT_HINT_TEXTSTYLE)
                    .size(SIZE_INPUT_HINT_TEXTSTYLE)
                    .color(color),
            );
        }

        column.spacing(SPACING_INPUT_COLUMN).into()
    }

    fn wrap_tooltip<'b>(
        &self,
        input_el: impl Into<Element<'b, Message>>,
        error_msg: Option<&'b str>,
        is_error: bool,
    ) -> Element<'b, Message> {
        let show = is_error && error_msg.is_some();
        Tooltip::new(input_el, error_msg.unwrap_or_default(), show).into()
    }
}
