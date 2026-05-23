//! A transparent wrapper widget that rasterizes its content into a
//! [`TextureCache`] once and then composites the cached texture every
//! frame, allowing cheap visual animations (translate / scale) without
//! re-rendering the underlying widget tree.
//!
//! # Caveats
//!
//! - Mouse / touch events are routed to the inner widget at its **original
//!   layout bounds**. The animation transform applied via [`Cached::transform`]
//!   only affects how the cached texture is composited; the hit-test box
//!   does **not** move with the animation. Use this widget for short visual
//!   transitions where event correctness during the animation is acceptable.
//!
//! - Cache invalidation is the caller's responsibility. Keep the
//!   [`TextureCache`] handle in your application state and call
//!   [`TextureCache::invalidate`] from `update()` whenever the
//!   underlying content has changed. Size changes are detected
//!   automatically by the backend.
use iced::Element;
use iced::Event;
use iced::Length;
use iced::Rectangle;
use iced::Size;
use iced::TextureCache;
use iced::Transformation;
use iced::Vector;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Shell, overlay};
use iced::mouse;

/// A wrapper widget that caches its content's rasterization in a
/// [`TextureCache`] and composites it under an optional
/// [`Transformation`].
///
/// See the [`module-level documentation`](self) for caveats.
pub struct Cached<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    cache: TextureCache,
    transform: Transformation,
}

impl<'a, Message, Theme, Renderer> Cached<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Wraps `content` in a `Cached` widget backed by `cache`.
    pub fn new(
        cache: TextureCache,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            cache,
            content: content.into(),
            transform: Transformation::IDENTITY,
        }
    }

    /// Sets the [`Transformation`] applied to the cached texture during
    /// compositing. Use this to animate translate / scale of the cached
    /// contents without re-rasterizing them.
    pub fn transform(mut self, transform: Transformation) -> Self {
        self.transform = transform;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Cached<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        let bounds = layout.bounds();
        let scale = renderer.scale_factor().unwrap_or(1.0);
        let size = Size::new(
            bounds.width.ceil().max(1.0) as u32,
            bounds.height.ceil().max(1.0) as u32,
        );

        // Record the content into the cache. If the cache is fresh and the
        // size/scale match, the closure is skipped entirely by the backend.
        let content = &self.content;
        let content_tree = &tree.children[0];
        renderer.draw_to_texture(&self.cache, size, scale, |r| {
            // Shift the content's coordinate origin to (0, 0) so it lands
            // inside the cache texture instead of being drawn at the
            // widget's screen-space position.
            r.with_translation(Vector::new(-bounds.x, -bounds.y), |r| {
                content.as_widget().draw(
                    content_tree, r, theme, style, layout, cursor, viewport,
                );
            });
        });

        // Composite the cache under the animation transform.
        renderer.with_transformation(self.transform, |r| {
            r.draw_cached_texture(&self.cache, bounds);
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
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
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Cached<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(value: Cached<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
