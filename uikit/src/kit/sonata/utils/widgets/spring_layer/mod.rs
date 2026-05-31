use iced::{
    Element, Event, Length, Rectangle, Size, Transformation, Vector,
    advanced::{
        Layout, Shell, Widget, layout, mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
    time::Instant,
    window,
};

use crate::kit::sonata::utils::animations::spring::{Spring, SpringParams};

// ─────────────────────────────────────────────────────────────────────────────
// Config
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct SpringLayerConfig {
    pub scale_pressed: f32,
    pub scale_released: f32,
    pub spring: SpringParams,
}

impl Default for SpringLayerConfig {
    fn default() -> Self {
        Self {
            scale_pressed: 0.9,
            scale_released: 1.0,
            spring: SpringParams::default(),
        }
    }
}

impl SpringLayerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scale_pressed(mut self, s: f32) -> Self {
        self.scale_pressed = s;
        self
    }

    pub fn scale_released(mut self, s: f32) -> Self {
        self.scale_released = s;
        self
    }

    pub fn bounce(mut self, b: f32) -> Self {
        self.spring.bounce = b;
        self
    }

    pub fn duration(mut self, d: f32) -> Self {
        self.spring.duration = d;
        self
    }

    pub fn spring(mut self, p: SpringParams) -> Self {
        self.spring = p;
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// State
// ─────────────────────────────────────────────────────────────────────────────

struct State {
    spring: Spring,
    is_pressed: bool,
    is_animating: bool,
    last_tick: Option<Instant>,
}

impl State {
    fn new(cfg: &SpringLayerConfig) -> Self {
        Self {
            spring: Spring::new(cfg.spring, cfg.scale_released),
            is_pressed: false,
            is_animating: false,
            last_tick: None,
        }
    }

    fn scaled_bounds(&self, bounds: Rectangle) -> Rectangle {
        let s = self.spring.position;
        let cx = bounds.x + bounds.width * 0.5;
        let cy = bounds.y + bounds.height * 0.5;
        Rectangle {
            x: cx + (bounds.x - cx) * s,
            y: cy + (bounds.y - cy) * s,
            width: bounds.width * s,
            height: bounds.height * s,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Widget
// ─────────────────────────────────────────────────────────────────────────────

pub struct SpringLayer<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    config: SpringLayerConfig,
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> SpringLayer<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            config: SpringLayerConfig::default(),
            content: content.into(),
        }
    }

    pub fn config(mut self, c: SpringLayerConfig) -> Self {
        self.config = c;
        self
    }

    pub fn scale_pressed(mut self, s: f32) -> Self {
        self.config.scale_pressed = s;
        self
    }

    pub fn scale_released(mut self, s: f32) -> Self {
        self.config.scale_released = s;
        self
    }

    pub fn bounce(mut self, b: f32) -> Self {
        self.config.spring.bounce = b;
        self
    }

    pub fn duration(mut self, d: f32) -> Self {
        self.config.spring.duration = d;
        self
    }

    pub fn spring(mut self, p: SpringParams) -> Self {
        self.config.spring = p;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SpringLayer<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(&self.config))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        if tree.children.is_empty() {
            tree.children.push(Tree::new(self.content.as_widget()));
        }

        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(child.size(), vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let child_layout = layout.children().next().unwrap();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if state.is_animating {
                let dt = state
                    .last_tick
                    .map(|prev| now.duration_since(prev).as_secs_f32())
                    .unwrap_or(1.0 / 60.0);

                state.last_tick = Some(*now);
                state.spring.tick(dt);

                if state.spring.is_settled() {
                    state.spring.snap();
                    state.is_animating = false;
                    state.last_tick = None;
                } else {
                    shell.request_redraw();
                }
            }

            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                child_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );
            return;
        }

        let scaled_bounds = state.scaled_bounds(layout.bounds());

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(scaled_bounds) =>
            {
                state.is_pressed = true;
                start_spring(state, self.config.scale_pressed, shell);
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.is_pressed => {
                state.is_pressed = false;
                start_spring(state, self.config.scale_released, shell);
            }
            _ => {}
        }

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

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let scale = state.spring.position;
        let bounds = layout.bounds();
        let cx = bounds.x + bounds.width * 0.5;
        let cy = bounds.y + bounds.height * 0.5;

        let draw_child = |renderer: &mut Renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );
        };

        if (scale - 1.0).abs() < 1e-4 {
            draw_child(renderer);
        } else {
            let t = Transformation::translate(cx, cy)
                * Transformation::scale(scale)
                * Transformation::translate(-cx, -cy);

            renderer.with_transformation(t, draw_child);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if cursor.is_over(state.scaled_bounds(layout.bounds())) {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::default()
        }
    }
}

fn start_spring<Message>(state: &mut State, target: f32, shell: &mut Shell<'_, Message>) {
    state.spring.set_target(target);
    state.is_animating = true;
    state.last_tick = None;
    shell.request_redraw();
}

impl<'a, Message, Theme, Renderer> From<SpringLayer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
{
    fn from(value: SpringLayer<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

pub fn spring_layer<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> SpringLayer<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    SpringLayer::new(content)
}
