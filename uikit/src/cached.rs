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

/// Controls whether the composited cache is snapped to the device-pixel grid.
///
/// Snapping the origin to whole device pixels avoids the bilinear resampling
/// that softens a cache composited at a fractional position, but quantizes
/// motion to whole device pixels. This is exactly what browsers do for
/// translation-only compositor layers to keep text crisp during `translate`
/// animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelSnap {
    /// Snap the origin to device pixels **only when the transform is a pure
    /// translation** (no scale / rotation) — the browser-style default. Pure
    /// translates stay crisp with zero extra cost; transforms that scale fall
    /// through to the resample path (pair them with [`Cached::supersample`]).
    #[default]
    Auto,
    /// Always snap the composited origin to the device-pixel grid.
    Always,
    /// Never snap; rely on [`Cached::supersample`] or accept the resampling
    /// softness of a fractional translate.
    Never,
}

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
    supersample: f32,
    pixel_snap: PixelSnap,
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
            supersample: 1.0,
            pixel_snap: PixelSnap::Auto,
        }
    }

    /// Sets the [`Transformation`] applied to the cached texture during
    /// compositing. Use this to animate translate / scale of the cached
    /// contents without re-rasterizing them.
    pub fn transform(mut self, transform: Transformation) -> Self {
        self.transform = transform;
        self
    }

    /// Records the cache at `factor`× the device resolution.
    ///
    /// When the cache is composited at a fractional device-pixel position
    /// (the usual case during a translate animation), bilinear sampling
    /// blends neighbouring texels and softens the image. Recording at a
    /// higher resolution shrinks that resampling error by roughly `1 /
    /// factor`, keeping motion both smooth **and** sharp at the cost of
    /// `factor²` texture memory and a one-time `factor²` rasterization.
    ///
    /// Values below `1.0` are clamped to `1.0` (the default). A factor of
    /// `1.5`–`2.0` is the sweet spot; larger factors undersample on the wgpu
    /// backend, which has no mipmaps for the cache. Changing the factor
    /// transparently re-records the cache.
    pub fn supersample(mut self, factor: f32) -> Self {
        self.supersample = factor.max(1.0);
        self
    }

    /// Sets the [`PixelSnap`] policy for compositing the cache.
    ///
    /// Defaults to [`PixelSnap::Auto`], which snaps the origin to the
    /// device-pixel grid whenever the [`transform`](Self::transform) is a pure
    /// translation — keeping `translate` animations crisp the way a browser
    /// does, with no extra memory. Use [`PixelSnap::Never`] together with
    /// [`supersample`](Self::supersample) if you prefer buttery sub-pixel
    /// motion over per-pixel sharpness.
    ///
    /// Note: the tiny_skia (CPU) backend always integer-snaps the origin, so
    /// this policy only changes behavior on wgpu.
    pub fn pixel_snap(mut self, mode: PixelSnap) -> Self {
        self.pixel_snap = mode;
        self
    }
}

/// Returns `true` if `t` is a pure 2D translation (identity linear part), the
/// case in which snapping the origin to the device grid is lossless. Reading
/// the raw matrix keeps this correct even if `Transformation` later gains a
/// `rotate` (rotation/shear leaves the off-diagonal terms non-zero).
fn is_translation_only(t: &Transformation) -> bool {
    let m: &[f32; 16] = t.as_ref();
    const EPS: f32 = 1e-4;
    (m[0] - 1.0).abs() < EPS
        && (m[5] - 1.0).abs() < EPS
        && m[1].abs() < EPS
        && m[4].abs() < EPS
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
        let ss = self.supersample.max(1.0);
        // Record at `scale * ss` so the texture (and the text inside it) is
        // rasterized at `ss`× the device resolution. The backend sizes the
        // backing store as `round(size * tex_scale)` and records through a
        // viewport at this scale, so supersampling needs no backend change.
        let tex_scale = scale * ss;
        let size = Size::new(
            bounds.width.ceil().max(1.0) as u32,
            bounds.height.ceil().max(1.0) as u32,
        );

        // Record the content into the cache. If the cache is fresh and the
        // size/scale match, the closure is skipped entirely by the backend.
        let content = &self.content;
        let content_tree = &tree.children[0];
        renderer.draw_to_texture(&self.cache, size, tex_scale, |r| {
            // Shift the content's coordinate origin to (0, 0) so it lands
            // inside the cache texture instead of being drawn at the
            // widget's screen-space position.
            r.with_translation(Vector::new(-bounds.x, -bounds.y), |r| {
                content.as_widget().draw(
                    content_tree, r, theme, style, layout, cursor, viewport,
                );
            });
        });

        // Composite the cache so the quad covers `physical / ss` device
        // pixels. The layout bounds are fractional; deriving the destination
        // from the physical backing size avoids the sub-pixel scale drift
        // that resamples and blurs the content. When `ss == 1` this is an
        // exact one-texel-per-device-pixel blit (scale 1.0); when `ss > 1` it
        // is a clean `ss`:1 downsample whose bilinear resampling stays sharp
        // even at a fractional translate.
        let physical = Size::new(
            (size.width as f32 * tex_scale).round(),
            (size.height as f32 * tex_scale).round(),
        );
        let cache_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: physical.width / tex_scale,
            height: physical.height / tex_scale,
        };

        // Optionally snap the composite origin to the device-pixel grid.
        // `Auto` snaps only a pure translation (browser behavior): the texels
        // then land exactly on device pixels, avoiding the bilinear resample
        // that would otherwise soften a fractional offset. A transform that
        // scales is left to the resample path (use `supersample`).
        let snap = match self.pixel_snap {
            PixelSnap::Always => true,
            PixelSnap::Never => false,
            PixelSnap::Auto => is_translation_only(&self.transform),
        };
        let transform = if snap {
            // Map the origin to device space, round it, and rebuild the
            // transform with a corrected translation (keeping any scale).
            let s = self.transform.scale_factor();
            let t = self.transform.translation();
            let dev_x = (s * bounds.x + t.x) * scale;
            let dev_y = (s * bounds.y + t.y) * scale;
            let tx = dev_x.round() / scale - s * bounds.x;
            let ty = dev_y.round() / scale - s * bounds.y;
            Transformation::translate(tx, ty) * Transformation::scale(s)
        } else {
            self.transform
        };

        renderer.with_transformation(transform, |r| {
            r.draw_cached_texture(&self.cache, cache_bounds);
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
