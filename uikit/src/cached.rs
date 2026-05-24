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
//!
//! - The widget reserves a small (~2 logical px) transparent margin around the
//!   content inside the cache texture so its edges anti-alias against
//!   transparency instead of crawling when composited at a fractional offset.
//!   Content is assumed to draw strictly within its layout bounds; the margin
//!   is bleed room, not extra paintable area.
use std::cell::Cell;

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
    /// Snap to the device grid **only for a pure translation that is at rest**
    /// — crisp when stopped, smooth while moving. The default.
    ///
    /// Snapping a *moving* element quantizes it to whole device pixels, which
    /// reads as jitter on a low-DPI display; resampling a *moving* element
    /// instead keeps it smooth. So `Auto` snaps a translation only once it
    /// stops changing (giving a pixel-perfect resting frame) and leaves it
    /// un-snapped while it animates (the composite shader's bicubic
    /// reconstruction keeps the moving edges from shimmering). Transforms that
    /// scale never snap — pair them with [`Cached::supersample`].
    #[default]
    Auto,
    /// Always snap the composited origin to the device-pixel grid.
    Always,
    /// Never snap; rely on [`Cached::supersample`] or accept the resampling
    /// softness of a fractional translate.
    Never,
}

/// Per-widget state, kept across frames by the widget tree.
#[derive(Default)]
struct State {
    /// The transform composited on the previous frame. Used to detect when the
    /// animation is at rest (transform unchanged) so [`PixelSnap::Auto`] can
    /// snap to the device grid for a crisp resting frame, while leaving motion
    /// un-snapped (and thus jitter-free). `Cell` so `draw` can update it through
    /// the shared `&State` reference.
    last_transform: Cell<Option<Transformation>>,
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
    auto_supersample_on_motion: bool,
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
            auto_supersample_on_motion: false,
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
    /// (the usual case during a translate animation), the composite's bicubic
    /// reconstruction blends neighbouring texels and softens the image. Recording at a
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
    /// Defaults to [`PixelSnap::Auto`]: a pure-translation cache is snapped to
    /// the device-pixel grid once it comes to rest (a crisp resting frame) and
    /// left un-snapped while moving (smooth, jitter-free motion, with the
    /// composite shader's bicubic reconstruction keeping moving edges from
    /// shimmering). See [`PixelSnap`] for the other policies.
    ///
    /// Note: the tiny_skia (CPU) backend always integer-snaps the origin, so
    /// this policy only changes behavior on wgpu.
    pub fn pixel_snap(mut self, mode: PixelSnap) -> Self {
        self.pixel_snap = mode;
        self
    }

    /// Enables supersampling **only while the cache is moving**.
    ///
    /// Off by default. When enabled, a cache that is animating and not snapped
    /// (the un-snapped path of [`PixelSnap::Auto`], or [`PixelSnap::Never`]) is
    /// recorded at `max(supersample, 1.5)`× the device resolution, so the
    /// fractional-offset resampling stays sharper in motion; at rest it drops
    /// back to the plain [`Cached::supersample`] factor (`1.0` by default) and
    /// snaps for a pixel-perfect resting frame.
    ///
    /// This re-records the cache at the rest↔motion transition (one extra
    /// rasterization of the content each way), so it trades a possible one-frame
    /// hitch at the start/end of an animation for sharper motion — without paying
    /// for supersampling permanently. Leave it off for genuinely expensive
    /// content whose re-rasterization cost outweighs the in-motion sharpness.
    pub fn auto_supersample_on_motion(mut self, on: bool) -> Self {
        self.auto_supersample_on_motion = on;
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
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
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
        // Transparent bleed margin (logical px) reserved around the content
        // inside the cache texture; applied where the texture is sized below.
        const MARGIN: u32 = 2;

        let bounds = layout.bounds();
        let scale = renderer.scale_factor().unwrap_or(1.0);

        // Detect whether the animation is at rest by comparing this frame's
        // transform to the previous one. This is *frame-perfect* detection (a
        // 4x4 matrix equality): if an animation curve plateaus on its very first
        // moving frame, `at_rest` reads `true` for that one frame and the
        // in-motion behavior (un-snap, optional supersample) engages a frame
        // late — self-correcting, and the same 1-frame property the snap below
        // already has. Real UI curves change the matrix every frame, so it is a
        // non-issue; add a `motion_frames` hysteresis counter only if false
        // triggers ever surface. The first draw is treated as at rest so a
        // never-animated widget is crisp immediately.
        let state = tree.state.downcast_ref::<State>();
        let at_rest = state
            .last_transform
            .get()
            .map_or(true, |prev| prev == self.transform);
        state.last_transform.set(Some(self.transform));

        // Choose the record resolution. `auto_supersample_on_motion` lifts it to
        // at least 1.5x while moving (sharper fractional-offset resampling) and
        // drops back to the base factor at rest, where we snap and want a 1:1
        // crisp blit. Changing the factor transparently re-records the cache.
        let base_ss = self.supersample.max(1.0);
        let ss = if self.auto_supersample_on_motion && !at_rest {
            base_ss.max(1.5)
        } else {
            base_ss
        };
        // Record at `scale * ss` so the texture (and the text inside it) is
        // rasterized at `ss`x the device resolution. The backend sizes the
        // backing store as `round(size * tex_scale)` and records through a
        // viewport at this scale, so supersampling needs no backend change.
        let tex_scale = scale * ss;

        // Reserve a transparent margin around the content inside the cache
        // texture. Without it the content's hard edges sit on the texture
        // boundary (texel 0 / N-1); composited at a fractional offset the
        // sampler's ClampToEdge can't fade the kernel into transparency and the
        // edge "crawls". Drawing the content inset by MARGIN surrounds it with
        // the transparent clear color so its edges anti-alias cleanly — the same
        // effect as wrapping it in a padded container, done automatically.
        // 2 logical px covers the 4x4 kernel's +/-2 texel reach at scale 1 / ss 1.
        let size = Size::new(
            bounds.width.ceil().max(1.0) as u32,
            bounds.height.ceil().max(1.0) as u32,
        );
        let padded = Size::new(size.width + 2 * MARGIN, size.height + 2 * MARGIN);

        // Record the content into the cache. If the cache is fresh and the
        // size/scale match, the closure is skipped entirely by the backend.
        let content = &self.content;
        let content_tree = &tree.children[0];
        renderer.draw_to_texture(&self.cache, padded, tex_scale, |r| {
            // Shift the content's coordinate origin so it lands inside the cache
            // texture inset by the transparent margin, instead of at the
            // widget's screen-space position.
            r.with_translation(
                Vector::new(-bounds.x + MARGIN as f32, -bounds.y + MARGIN as f32),
                |r| {
                    content.as_widget().draw(
                        content_tree, r, theme, style, layout, cursor, viewport,
                    );
                },
            );
        });

        // Composite the (padded) cache. The layout bounds are fractional;
        // deriving the destination size from the physical backing size avoids
        // the sub-pixel scale drift that resamples and blurs the content. The
        // origin is shifted back by the margin so the content lands exactly
        // where it would without the margin. When `ss == 1` this is an exact
        // one-texel-per-device-pixel blit; when `ss > 1` it is a clean `ss`:1
        // downsample whose resampling stays sharp even at a fractional translate.
        let physical = Size::new(
            (padded.width as f32 * tex_scale).round(),
            (padded.height as f32 * tex_scale).round(),
        );
        let cache_bounds = Rectangle {
            x: bounds.x - MARGIN as f32,
            y: bounds.y - MARGIN as f32,
            width: physical.width / tex_scale,
            height: physical.height / tex_scale,
        };

        // Decide whether to snap the composite to the device-pixel grid. `Auto`
        // snaps only a pure translation that is **at rest**: the texels then land
        // 1:1 on device pixels and the cache is pixel-perfect crisp (the shader
        // takes its aligned fast path). While moving, `Auto` leaves the offset
        // fractional so motion stays smooth and jitter-free — the shader's
        // bicubic reconstruction keeps the moving edges sharp. A transform that
        // scales never snaps (pair it with `supersample`).
        let snap = match self.pixel_snap {
            PixelSnap::Always => true,
            PixelSnap::Never => false,
            PixelSnap::Auto => is_translation_only(&self.transform) && at_rest,
        };
        let transform = if snap {
            // Snap the *texture/quad origin* (the cache_bounds origin), not the
            // content origin, so the whole texel grid lands on the device grid
            // and the resting frame samples at integer phase, where Catmull-Rom
            // is exact. Snapping the content origin instead would only align the
            // grid when MARGIN*scale is integral (e.g. scale 1); at a fractional
            // scale it would leave every sample at ~0.5 phase and soften the
            // resting frame. At scale 1 the two are identical. Any scale is kept.
            let s = self.transform.scale_factor();
            let t = self.transform.translation();
            let qx = bounds.x - MARGIN as f32;
            let qy = bounds.y - MARGIN as f32;
            let dev_x = (s * qx + t.x) * scale;
            let dev_y = (s * qy + t.y) * scale;
            let tx = dev_x.round() / scale - s * qx;
            let ty = dev_y.round() / scale - s * qy;
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
