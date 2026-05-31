use iced::{
    Color, Degrees, Element, Gradient, Length, Padding, Rectangle, Size,
    advanced::{Widget, layout, overlay, renderer::Quad, widget::Tree},
    gradient::Linear,
};

use crate::kit::sonata::components::sidebar::vars::COLOR_SIDEBAR_BACKGROUND;

pub struct Sidebar<'a, Message, Theme, Renderer> {
    child: Vec<Element<'a, Message, Theme, Renderer>>,
    width: Length,
    height: Length,
}

impl<'a, Message, Theme, Renderer> Sidebar<'a, Message, Theme, Renderer>
where
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
            height: Length::Fill,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Sidebar<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        Size::new(self.width, iced::Length::Fill)
    }

    // fn state(&self) -> iced::advanced::widget::tree::State {
    //     tree::State::new(State::default())
    // }

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
        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            self.width,
            self.height,
            Padding::from(10),
            5.0,
            iced::Alignment::Start,
            &mut self.child,
            &mut tree.children,
        )
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
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(sidebar: Sidebar<'a, Message, Theme, Renderer>) -> Self {
        Self::new(sidebar)
    }
}
