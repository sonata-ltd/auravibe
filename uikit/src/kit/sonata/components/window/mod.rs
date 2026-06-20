use iced::{
    Background, Border, Color, Element,
    Length::{Fill, Shrink},
    Theme, Vector,
    border::Radius,
    widget::{Space, column, container, text},
};

use crate::{
    cached::Cached,
    definition::window::UiWindow,
    kit::sonata::{
        Sonata, components::content_stack::widget::ContentStack, utils::text::text::TextStyle,
    },
};

mod vars;

impl<Message: Clone + 'static> Sonata<Message> {
    pub fn window<'a>(&self, params: UiWindow<'a, Message>) -> Element<'a, Message> {
        let mut widget =
            ContentStack::from_vec(params.content_stack_childs.1, params.content_stack_childs.0);

        // if let Some(max_height) = params.max_height {
        //     widget = widget.max_height(max_height);
        // }

        if let Some(width) = params.width {
            widget = widget.width(width);
        }

        let header = Cached::new(
            params.header_texture_cache.clone(),
            container(
                text(params.label)
                    .size(18)
                    .font(TextStyle::TextLgSm.build_font()),
            )
            .style(|_: &Theme| container::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: *vars::HEADER_BORDER,
                shadow: *vars::HEADER_BOTTOM_SHADOW,
                ..Default::default()
            })
            .width(Fill)
            .padding(*vars::HEADER_PADDING),
        );

        // let content = container(
        //     scrollable(
        //         container(column![
        //             params.child,
        //             if params.controls_child.is_some() {
        //                 space().height(64)
        //             } else {
        //                 space().height(0)
        //             },
        //             space().height(100)
        //         ])
        //         .width(Fill),
        //     )
        //     .style(custom_scrollable_style)
        //     .direction(Direction::Vertical(
        //         Scrollbar::new().width(5).scroller_width(5),
        //     )),
        // )
        // .style(|_| container::Style {
        //     background: None,
        //     border: Border {
        //         color: Color::TRANSPARENT,
        //         width: 0.0,
        //         radius: Radius::from(0.0),
        //     },
        //     ..Default::default()
        // })
        // .height(Shrink)
        // .width(Fill)
        // .padding(Padding {
        //     top: 5.0,
        //     left: 0.0,
        //     right: 5.0,
        //     bottom: 5.0,
        // });

        let main_content = column![header, widget, {
            let mut controls: Element<'_, Message> = Space::new().into();

            if let Some(controls_child) = params.controls_child {
                controls = controls_child;
            }

            controls
        }]
        .width(Fill)
        .clip(true);

        // let controls_overlay = if let Some(controls) = params.controls_child {
        //     container(
        //         container(controls)
        //             .style(|_| container::Style {
        //                 background: *vars::BACKGROUND,
        //                 border: Border {
        //                     color: Color::TRANSPARENT,
        //                     width: 0.0,
        //                     radius: *vars::CONTROLS_RADIUS,
        //                 },
        //                 ..Default::default()
        //             })
        //             .height(64)
        //             .width(Fill)
        //             .padding(*vars::CONTROLS_PADDING),
        //     )
        //     .align_x(Horizontal::Center)
        //     .align_y(Vertical::Bottom)
        //     .width(Fill)
        //     .height(Fill)
        // } else {
        //     container(Space::new()).width(Fill).height(Fill)
        // };

        // let window_content = stack![main_content, controls_overlay];

        let window = container(
            container(main_content)
                .style(|_| container::Style {
                    background: *vars::BACKGROUND,
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: Radius::from(25),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba8(0, 0, 0, 0.6),
                        offset: Vector::ZERO,
                        blur_radius: 1.0,
                    },
                    ..Default::default()
                })
                .height(Shrink)
                .width(400)
                .clip(true),
        )
        .style(|_| container::Style {
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(25),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.25),
                offset: Vector { x: 0.0, y: 38.0 },
                blur_radius: 90.0,
            },
            ..Default::default()
        })
        .clip(true);

        // let window_wrapper = stack![if let Some(close_event) = params.wrapper_close_event {
        //     container(opaque(
        //         mouse_area(
        //             center(opaque(window))
        //                 .style(|_| container::Style {
        //                     background: Some(
        //                         Color {
        //                             a: 0.4,
        //                             ..Color::BLACK
        //                         }
        //                         .into(),
        //                     ),
        //                     ..Default::default()
        //                 })
        //                 .padding(15),
        //         )
        //         .on_press(close_event),
        //     ))
        // } else {
        //     window
        // }];

        // window_wrapper.into()

        window.into()
    }
}
