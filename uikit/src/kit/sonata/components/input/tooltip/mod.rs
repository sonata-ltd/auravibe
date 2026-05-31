use iced::{
    Color, Element, Length, Size, Transformation,
    advanced::{
        Widget,
        layout::{Limits, Node},
        renderer::Quad,
        svg,
        text::{Paragraph, paragraph::Plain},
        widget::{Tree, text, tree},
    },
    border::Radius,
};

use crate::kit::sonata::components::input::tooltip::vars::{
    COLOR_TOOLTIP_BACKGROUND, COLOR_TOOLTIP_TEXT, FONT_TOOLTIP_TEXT, SIZE_TOOLTIP_ARROW_HEIGHT,
    SIZE_TOOLTIP_ARROW_RIGHT_OFFSET, SIZE_TOOLTIP_GAP, SIZE_TOOLTIP_HORIZONTAL_PADDING,
    SIZE_TOOLTIP_RIGHT_OFFSET, SIZE_TOOLTIP_TEXT, SIZE_TOOLTIP_VERTICAL_PADDING,
};

mod draw;
mod vars;

pub struct Tooltip<'a, Message, Theme, Renderer> {
    child: Element<'a, Message, Theme, Renderer>,
    text: String,
    show: bool,
}

struct TooltipState<P: iced::advanced::text::Paragraph>(Plain<P>);

impl<'a, Message, Theme, Renderer> Tooltip<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(
        child: impl Into<Element<'a, Message, Theme, Renderer>>,
        text: impl Into<String>,
        show: bool,
    ) -> Self {
        Self {
            child: child.into(),
            text: text.into(),
            show,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tooltip<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer<Font = iced::Font> + svg::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        self.child.as_widget().size()
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        tree::Tag::of::<TooltipState<<Renderer as iced::advanced::text::Renderer>::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TooltipState(Plain::<
            <Renderer as iced::advanced::text::Renderer>::Paragraph,
        >::default()))
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![Tree::new(&self.child)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.child));
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let state = tree
            .state
            .downcast_mut::<TooltipState<<Renderer as iced::advanced::text::Renderer>::Paragraph>>(
            );

        let tooltip_limits = Limits::new(
            Size::ZERO,
            Size {
                width: limits.max().width,
                height: limits.max().height,
            },
        )
        .shrink(Size {
            width: SIZE_TOOLTIP_HORIZONTAL_PADDING * 2.0,
            height: SIZE_TOOLTIP_VERTICAL_PADDING * 2.0,
        });

        iced::advanced::widget::text::layout(
            &mut state.0,
            renderer,
            &tooltip_limits,
            &self.text,
            iced::advanced::widget::text::Format {
                width: Length::Shrink,
                height: Length::Shrink,
                size: Some(SIZE_TOOLTIP_TEXT.into()),
                font: Some(FONT_TOOLTIP_TEXT),
                line_height: iced::widget::text::LineHeight::default(),
                align_x: iced::widget::text::Alignment::Default,
                align_y: iced::alignment::Vertical::Top,
                shaping: iced::widget::text::Shaping::Basic,
                wrapping: iced::widget::text::Wrapping::None,
                ..Default::default()
            },
        );

        let child_node = self
            .child
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        Node::with_children(child_node.size(), vec![child_node])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.child.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );

        if !self.show {
            return;
        }

        let state = tree
            .state
            .downcast_ref::<TooltipState<<Renderer as iced::advanced::text::Renderer>::Paragraph>>(
            );

        // Calc size of paragrath
        let text_size = Paragraph::min_bounds(state.0.raw());
        let tooltip_w = text_size.width + SIZE_TOOLTIP_HORIZONTAL_PADDING * 2.0;
        let tooltip_h = text_size.height + SIZE_TOOLTIP_VERTICAL_PADDING * 2.0;

        let input_bounds = layout.bounds();

        let space_above = input_bounds.y - viewport.y;
        let space_below = (viewport.y + viewport.height) - (input_bounds.y + input_bounds.height);

        let gap = (SIZE_TOOLTIP_ARROW_HEIGHT + SIZE_TOOLTIP_GAP).round();
        let tooltip_y = if space_above >= tooltip_h + gap {
            input_bounds.y - tooltip_h - gap
        } else if space_below >= tooltip_h + gap {
            input_bounds.y + input_bounds.height + gap
        } else {
            // No space at all
            if space_above >= space_below {
                viewport.y
            } else {
                viewport.y + viewport.height - tooltip_h
            }
        };

        // Snap to right edge of child inside viewport
        let tooltip_x = input_bounds.x + input_bounds.width - tooltip_w - SIZE_TOOLTIP_RIGHT_OFFSET;

        let tooltip_bounds = iced::Rectangle {
            x: tooltip_x,
            y: tooltip_y,
            width: tooltip_w,
            height: tooltip_h,
        };

        let text_bounds = iced::Rectangle {
            x: tooltip_bounds.x + SIZE_TOOLTIP_HORIZONTAL_PADDING,
            y: tooltip_bounds.y + SIZE_TOOLTIP_VERTICAL_PADDING,
            width: text_size.width,
            height: text_size.height,
        };

        renderer.with_layer(*viewport, |renderer| {
            renderer.fill_quad(
                Quad {
                    bounds: tooltip_bounds,
                    border: iced::Border {
                        radius: Radius {
                            top_left: 10.0,
                            top_right: 10.0,
                            bottom_left: 10.0,
                            bottom_right: 6.0,
                        },
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                },
                COLOR_TOOLTIP_BACKGROUND,
            );

            let arrow_position = iced::Point::new(
                tooltip_bounds.x + tooltip_bounds.width
                    - SIZE_TOOLTIP_ARROW_HEIGHT
                    - SIZE_TOOLTIP_ARROW_RIGHT_OFFSET,
                tooltip_bounds.y + tooltip_bounds.height,
            );

            draw::tooltip_arrow(renderer, arrow_position, *viewport);

            renderer.with_transformation(
                Transformation::translate(text_bounds.x, text_bounds.y),
                |renderer| {
                    iced::advanced::widget::text::draw(
                        renderer,
                        style,
                        iced::Rectangle {
                            x: 0.0,
                            y: 0.0,
                            width: text_size.width,
                            height: text_size.height,
                        },
                        state.0.raw(),
                        text::Style {
                            color: Some(COLOR_TOOLTIP_TEXT),
                        },
                        viewport,
                    );
                },
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        self.child.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.child.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Tooltip<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::text::Renderer<Font = iced::Font> + svg::Renderer + 'a,
{
    fn from(value: Tooltip<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
