use iced::{
    Color, Degrees, Element, Event, Gradient, Length, Padding, Rectangle, Size, Vector,
    advanced::{
        Widget, layout, overlay,
        renderer::Quad,
        svg,
        widget::{Tree, tree},
    },
    gradient::Linear,
    mouse, window,
};

use crate::kit::sonata::{
    components::sidebar::vars::COLOR_SIDEBAR_BACKGROUND,
    utils::animations::{
        Animation,
        spring::{Spring, SpringParams},
        tick_animation,
    },
};

mod icons;

const HEADER_HEIGHT: f32 = 44.0;

pub struct Sidebar<'a, Message, Theme, Renderer> {
    child: Vec<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    min_width: f32,
    height: Length,
    collapsed: bool,
    header_height: f32,
    show_header: bool,
    on_toggle: Option<Message>,
}

#[derive(Debug)]
pub struct State {
    collapsed: bool,
    animation: Animation,
}

impl State {
    pub fn new(collapsed: bool) -> Self {
        let desired = if collapsed { 1.0 } else { 0.0 };

        Self {
            collapsed,
            animation: Animation {
                spring: Spring::new(SpringParams::default(), desired),
                is_animating: false,
                last_tick: None,
            },
        }
    }
}

impl<'a, Message, Theme, Renderer> Sidebar<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self::from_vec(Vec::with_capacity(cap))
    }

    pub fn with_children(
        child: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let iterator = child.into_iter();
        Self::with_capacity(iterator.size_hint().0).extend_childs(iterator)
    }

    pub fn extend_childs(
        self,
        child: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        child.into_iter().fold(self, Self::push)
    }

    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let child = child.into();
        let child_size = child.as_widget().size_hint();

        if !child_size.is_void() {
            self.width = self.width.enclose(child_size.width);
            self.child.push(child);
        }

        self
    }

    pub fn from_vec(child: Vec<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            child,
            width: Length::Shrink,
            min_width: 50.0,
            height: Length::Fill,
            collapsed: false,
            header_height: HEADER_HEIGHT,
            show_header: false,
            on_toggle: None,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn show_header(mut self) -> Self {
        self.show_header = true;
        self
    }

    pub fn on_toggle(mut self, msg: Message) -> Self {
        self.on_toggle = Some(msg);
        self
    }

    fn get_header_height(&self) -> f32 {
        if self.show_header {
            self.header_height
        } else {
            0.0
        }
    }

    fn toggle_bounds(&self, bounds: Rectangle) -> Rectangle {
        let icon_size = 20.0;
        let pad = (self.header_height - icon_size) / 2.0;

        Rectangle {
            x: bounds.x + pad,
            y: bounds.y + pad,
            width: icon_size,
            height: icon_size,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Sidebar<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer + iced::advanced::svg::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        Size::new(self.width, iced::Length::Fill)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        tree::State::new(State::new(self.collapsed))
    }

    fn children(&self) -> Vec<Tree> {
        self.child.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.child);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.child
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            tick_animation(&mut state.animation, now, shell, None::<fn()>);
        }

        if self.show_header {
            if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
                let rect = self.toggle_bounds(layout.bounds());

                if cursor.is_over(rect) {
                    state.collapsed = !state.collapsed;

                    let target = if state.collapsed { 1.0 } else { 0.0 };
                    state.animation.spring.set_target(target);
                    state.animation.is_animating = true;
                    state.animation.last_tick = None;

                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();

                    if let Some(msg) = self.on_toggle.clone() {
                        shell.publish(msg);
                    }

                    return;
                }
            }
        }

        for ((child, tree), layout) in self
            .child
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .update(tree, event, layout, cursor, renderer, shell, viewport);
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let state = tree.state.downcast_mut::<State>();

        let full_width = limits
            .resolve(self.width, self.height, Size::new(self.min_width, 0.0))
            .width;
        let spring_pos = state.animation.spring.position.clamp(0.0, 1.0);
        let width = interpolate(full_width, self.min_width, spring_pos);

        let current_header_height = self.get_header_height();

        let body_limits = layout::Limits::new(
            Size::new(
                limits.min().width,
                (limits.min().height - current_header_height).max(0.0),
            ),
            Size::new(
                limits.max().width,
                (limits.max().height - current_header_height).max(0.0),
            ),
        );

        let body = layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &body_limits,
            width.into(),
            self.height,
            Padding::from(10),
            5.0,
            iced::Alignment::Start,
            &mut self.child,
            &mut tree.children,
        );

        let body_h = body.size().height;

        let children: Vec<layout::Node> = body
            .children()
            .iter()
            .cloned()
            .map(|n| n.translate(Vector::new(0.0, current_header_height)))
            .collect();
        layout::Node::with_children(Size::new(width, current_header_height + body_h), children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        // Draw background
        renderer.fill_quad(
            Quad {
                bounds,
                ..Default::default()
            },
            COLOR_SIDEBAR_BACKGROUND,
        );

        // Draw side shadow
        renderer.fill_quad(
            Quad {
                bounds,
                ..Default::default()
            },
            Gradient::Linear(
                Linear::new(Degrees::from(90))
                    .add_stop(0.95, Color::from_rgba(0.0, 0.0, 0.0, 0.0))
                    .add_stop(1.0, Color::from_rgba(0.0, 0.0, 0.0, 0.03)),
            ),
        );

        if self.show_header {
            let rect = self.toggle_bounds(bounds);

            if cursor.is_over(rect) {
                renderer.fill_quad(
                    Quad {
                        bounds: rect,
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                );
            }

            renderer.draw_svg(
                svg::Svg {
                    handle: icons::chevron(state.collapsed),
                    color: None,
                    rotation: 0.0.into(),
                    opacity: 1.0,
                },
                rect,
                *viewport,
            );
        }

        for ((child, tree), layout) in self
            .child
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().intersects(viewport))
        {
            child
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        if self.show_header && cursor.is_over(self.toggle_bounds(layout.bounds())) {
            return mouse::Interaction::Pointer;
        }

        self.child
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: layout::Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.child,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Sidebar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::svg::Renderer + 'a,
{
    fn from(sidebar: Sidebar<'a, Message, Theme, Renderer>) -> Self {
        Self::new(sidebar)
    }
}

#[inline]
fn interpolate(current: f32, next: f32, pos: f32) -> f32 {
    current + (next - current) * pos
}
