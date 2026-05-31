use std::slice::from_ref;

use iced::{
    Background, Color, Element, Event, Length, Padding, Point, Rectangle, Size, Vector,
    advanced::{
        Widget, layout,
        widget::{self, tree},
    },
    mouse, touch,
};

mod draw;
mod focus;
mod types;

pub use types::{Appearance, BorderLayer, BorderSide, Status};

#[derive(Debug, Default)]
struct State {
    is_pressed: bool,
    is_focused: bool,
    is_hovered: bool,
}

pub struct MultiBorder<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    layers: Vec<BorderLayer>,
    background: Option<Background>,
    style: Option<Box<dyn Fn(&Theme, Status) -> Appearance + 'a>>,
    is_disabled: bool,
    is_focused: bool,
    width: Option<Length>,
    height: Option<Length>,
    reserved_outer: Option<f32>,
    focus_detector: focus::FocusDetector,
}

impl<'a, Message, Theme, Renderer> MultiBorder<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            layers: Vec::new(),
            background: None,
            style: None,
            is_disabled: false,
            is_focused: false,
            width: None,
            height: None,
            reserved_outer: None,
            focus_detector: focus::FocusDetector::default(),
        }
    }

    // Border layers
    pub fn outer_border(mut self, width: f32, color: Color) -> Self {
        self.layers.push(BorderLayer::outer(width, color));
        self
    }

    pub fn inner_border(mut self, width: f32, color: Color) -> Self {
        self.layers.push(BorderLayer::inner(width, color));
        self
    }

    pub fn outer_border_radius(mut self, width: f32, color: Color, radius: f32) -> Self {
        self.layers
            .push(BorderLayer::outer(width, color).radius(radius));
        self
    }

    pub fn inner_border_radius(mut self, width: f32, color: Color, radius: f32) -> Self {
        self.layers
            .push(BorderLayer::inner(width, color).radius(radius));
        self
    }

    /// Sets `offset` of the last pushed layer. Call right after the border method.
    pub fn border_offset(mut self, offset: f32) -> Self {
        if let Some(last) = self.layers.last_mut() {
            last.offset = offset;
        }
        self
    }

    /// Sets `radius` on the last pushed layer. Call right after the border method.
    pub fn border_radius(mut self, radius: f32) -> Self {
        if let Some(last) = self.layers.last_mut() {
            last.radius = radius;
        }
        self
    }

    // Appearance
    pub fn background(mut self, bg: impl Into<Background>) -> Self {
        self.background = Some(bg.into());
        self
    }

    // Dynamic styling
    // When set, `layers` and `background` fields are ignored
    pub fn style(mut self, f: impl Fn(&Theme, Status) -> Appearance + 'a) -> Self {
        self.style = Some(Box::new(f));
        self
    }

    pub fn disabled(mut self, state: bool) -> Self {
        self.is_disabled = state;
        self
    }

    pub fn focused(mut self, state: bool) -> Self {
        self.is_focused = state;
        self
    }

    // Layout
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Reserve a fixed outer padding for borders instead of computing it automatically.
    /// Useful if you want the layout to stay stable across style changes.
    pub fn reserve_outer(mut self, padding: f32) -> Self {
        self.reserved_outer = Some(padding);
        self
    }

    // Internal helpers
    fn default_outer_thickness(&self) -> f32 {
        self.layers
            .iter()
            .filter(|l| l.side == BorderSide::Outer)
            .map(|l| l.width + l.offset)
            .sum()
    }

    fn layout_outer_thickness(&self) -> f32 {
        self.reserved_outer
            .unwrap_or_else(|| self.default_outer_thickness())
    }

    fn current_status(&self, state: &State) -> Status {
        Status {
            is_hovered: state.is_hovered,
            is_pressed: state.is_pressed,
            is_focused: if self.is_disabled {
                false
            } else {
                state.is_focused
            },
            is_disabled: self.is_disabled,
        }
    }

    fn resolve_appearance(&self, theme: &Theme, status: Status) -> Appearance {
        if let Some(style_fn) = &self.style {
            style_fn(theme, status)
        } else {
            Appearance {
                layers: self.layers.clone(),
                background: self.background,
            }
        }
    }
}

// Widget impl
impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MultiBorder<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(from_ref(&self.content));
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let outer = self.layout_outer_thickness();

        // Outer borders shrink available space
        // Inner borders are drawn on top (no shink)
        let inner_limits = limits.shrink(Padding::from(outer));

        let content_node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &inner_limits)
            .move_to(Point::new(outer, outer));

        let content_size = content_node.size();
        let total_size = Size::new(
            content_size.width + 2.0 * outer,
            content_size.height + 2.0 * outer,
        );

        layout::Node::with_children(total_size, vec![content_node])
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let status = self.current_status(state);
        let appearance = self.resolve_appearance(theme, status);

        let bounds = layout.bounds();
        let reserved = self.layout_outer_thickness();

        let content_bounds = Rectangle {
            x: bounds.x + reserved,
            y: bounds.y + reserved,
            width: bounds.width - 2.0 * reserved,
            height: bounds.height - 2.0 * reserved,
        };

        draw::outer_borders(renderer, &appearance.layers, content_bounds);
        draw::background_full(renderer, &appearance, content_bounds);
        draw::content(self, tree, renderer, theme, style, layout, cursor, viewport);
        draw::inner_borders(renderer, &appearance.layers, content_bounds);
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &iced::Event,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Some(child_layout) = layout.children().next() {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                child_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );
        }

        let is_over = cursor.is_over(layout.bounds());
        let new_focus = self.resolve_focus(tree, Some(is_over));
        let state = tree.state.downcast_mut::<State>();

        let snapshot = (state.is_pressed, state.is_focused, state.is_hovered);

        match &event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.is_pressed = is_over;
                state.is_focused = is_over;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.is_pressed = false;
            }
            Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                let contains = layout.bounds().contains(*position);
                state.is_focused = contains;
                state.is_pressed = contains;
            }
            Event::Touch(touch::Event::FingerLifted { .. }) => {
                state.is_pressed = false;
            }
            _ => {}
        }

        state.is_focused = new_focus;

        let new_shapshot = (state.is_pressed, state.is_focused, state.is_hovered);
        if snapshot != new_shapshot {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        if let Some(child_layout) = layout.children().next() {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                child_layout,
                cursor,
                viewport,
                renderer,
            )
        } else {
            iced::advanced::mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: layout::Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<MultiBorder<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn from(
        value: MultiBorder<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(value)
    }
}

pub trait MultiBorderExt<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn outer_border(self, width: f32, color: Color) -> MultiBorder<'a, Message, Theme, Renderer>;
    fn inner_border(self, width: f32, color: Color) -> MultiBorder<'a, Message, Theme, Renderer>;
    fn outer_border_radius(
        self,
        width: f32,
        color: Color,
        radius: f32,
    ) -> MultiBorder<'a, Message, Theme, Renderer>;
    fn inner_border_radius(
        self,
        width: f32,
        color: Color,
        radius: f32,
    ) -> MultiBorder<'a, Message, Theme, Renderer>;

    fn styled_border(
        self,
        f: impl Fn(&Theme, Status) -> Appearance + 'a,
    ) -> MultiBorder<'a, Message, Theme, Renderer>;
}

impl<'a, Message, Theme, Renderer, W> MultiBorderExt<'a, Message, Theme, Renderer> for W
where
    W: Into<Element<'a, Message, Theme, Renderer>>,
    Renderer: iced::advanced::Renderer + 'a,
    Message: 'a,
    Theme: 'a,
{
    fn outer_border(self, width: f32, color: Color) -> MultiBorder<'a, Message, Theme, Renderer> {
        MultiBorder::new(self).outer_border(width, color)
    }
    fn inner_border(self, width: f32, color: Color) -> MultiBorder<'a, Message, Theme, Renderer> {
        MultiBorder::new(self).inner_border(width, color)
    }
    fn outer_border_radius(
        self,
        width: f32,
        color: Color,
        radius: f32,
    ) -> MultiBorder<'a, Message, Theme, Renderer> {
        MultiBorder::new(self).outer_border_radius(width, color, radius)
    }
    fn inner_border_radius(
        self,
        width: f32,
        color: Color,
        radius: f32,
    ) -> MultiBorder<'a, Message, Theme, Renderer> {
        MultiBorder::new(self).inner_border_radius(width, color, radius)
    }

    fn styled_border(
        self,
        f: impl Fn(&Theme, Status) -> Appearance + 'a,
    ) -> MultiBorder<'a, Message, Theme, Renderer> {
        MultiBorder::new(self).style(f)
    }
}

// Constructor
pub fn multiborder<'a, Message, Theme, Renderer>(
    el: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> MultiBorder<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + 'a,
{
    MultiBorder::new(el)
}
