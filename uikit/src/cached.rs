//! A transparent wrapper widget that rasterizes its content into a
//! [`TextureCache`] once and then composites the cached texture every
//! frame, allowing cheap visual animations (translate / scale) without
//! re-rendering the underlying widget tree.
//!
//! # Compositing modes
//!
//! [`CompositingMode::Layer`] (the default) registers a compositor layer
//! with the runtime via [`Shell::register_layer`]. The actual composite
//! quad is emitted in a separate pass after the widget-tree draw walk
//! completes, so nested animated widgets compose correctly — an inner
//! `Cached`'s transform applies on top of (left-multiplied by) its
//! outer's transform, **without** baking the inner's current pose into
//! the outer's texture.
//!
//! [`CompositingMode::Bake`] preserves the pre-v8 behavior: the
//! composite is emitted inline at the end of [`Widget::draw`], in
//! source order with surrounding siblings. Pick this when source-order
//! painting matters more than nested-animation correctness.
//!
//! # Caveats
//!
//! - Mouse / touch events are routed to the inner widget at its
//!   **original layout bounds**. The animation transform applied via
//!   [`Cached::transform`] only affects how the cached texture is
//!   composited; the hit-test box does **not** move with the animation.
//!   Use this widget for short visual transitions where event
//!   correctness during the animation is acceptable.
//!
//! - Cache invalidation is the caller's responsibility. Keep the
//!   [`TextureCache`] handle in your application state and call
//!   [`TextureCache::invalidate`] from `update()` whenever the
//!   underlying content has changed. Size changes are detected
//!   automatically by the backend.
//!
//! - The widget reserves a small (~2 logical px) transparent margin
//!   around the content inside the cache texture so its edges
//!   anti-alias against transparency instead of crawling when
//!   composited at a fractional offset. Content is assumed to draw
//!   strictly within its layout bounds; the margin is bleed room, not
//!   extra paintable area.
//!
//! - **z-order in `Layer` mode**: a `Cached` registered as a compositor
//!   layer paints on top of any non-`Cached` content rendered before
//!   it in the same compose pass. If a non-`Cached` sibling needs to
//!   paint *over* the `Cached` in source order, switch the `Cached` to
//!   [`CompositingMode::Bake`] or wrap the sibling in a `Cached` as
//!   well so both participate in the compose pass.
use std::cell::Cell;
use std::sync::Arc;

use iced::Element;
use iced::Event;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Size;
use iced::TextureCache;
use iced::Transformation;
use iced::Vector;
use iced::advanced::graphics::core::TextureRecordMode;
use iced::advanced::layer::LayerSlot;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Shell, overlay};
use iced::mouse;
use iced::window;

/// Transparent bleed margin (logical px) reserved around the content
/// inside the cache texture. Promoted to a module-level const so it is
/// referenced by both `update` and `draw` from the same source of
/// truth.
pub const MARGIN: u32 = 2;

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
    /// Snap the layout-bounds portion of the composite origin to the
    /// device-pixel grid; leave the user-supplied transform's translate
    /// component fractional.
    ///
    /// Designed for the case where the cached widget's `bounds.x` /
    /// `bounds.y` shift discretely (e.g. the surrounding layout reshuffles
    /// during a window resize) **while** the widget is also animating via
    /// [`Cached::transform`]. With [`Auto`] or [`Never`], the changing
    /// bounds add a discrete component to the bicubic-resampling phase,
    /// making the blur level visibly "breathe" frame-to-frame. `Hybrid`
    /// removes that contribution — the resampling phase varies only with
    /// the smoothly-changing user transform, so perceived blur stays
    /// stable across layout-shuffle frames.
    ///
    /// Trade-off: the displayed cache origin can shift by up to
    /// **0.5 logical pixels** (½ device pixel) from the layout-intended
    /// position when `bounds.x` crosses a pixel boundary. Bounded,
    /// predictable, and usually imperceptible at typical DPI.
    ///
    /// [`Auto`]: PixelSnap::Auto
    /// [`Never`]: PixelSnap::Never
    Hybrid,
}

/// Selects how `Cached` composites its texture into the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompositingMode {
    /// Register a compositor layer; the composite quad is emitted in a
    /// separate pass after the widget-tree draw walk, with parent
    /// transforms chained and ancestor clips respected.
    ///
    /// Required for correct nested animation: an inner `Cached`'s
    /// transform applies on top of (left-multiplied by) its outer's,
    /// without baking the inner's current pose into the outer's
    /// texture.
    ///
    /// **z-order**: source-order painting is **not** preserved relative
    /// to non-`Cached` siblings — see the module docs.
    #[default]
    Layer,
    /// Composite inline at the end of `draw`, in source order with
    /// surrounding siblings. Nested-animation correctness is the
    /// caller's responsibility (typically: don't animate a nested
    /// `Cached` in `Bake` mode).
    Bake,
}

/// Per-widget state, kept across frames by the widget tree.
struct State {
    /// Persistent compositor-layer slot. Created in [`Widget::state`]
    /// and re-keyed in [`Widget::diff`] when `self.cache` identity
    /// changes. Cloned into [`Shell::register_layer`] each frame; the
    /// runtime drops every clone when it clears the registry.
    slot: Arc<LayerSlot>,
    /// The transform composited on the previous frame. Used to detect
    /// when the animation is at rest (transform unchanged) so
    /// [`PixelSnap::Auto`] can snap to the device grid for a crisp
    /// resting frame, while leaving motion un-snapped (and thus
    /// jitter-free).
    last_transform: Cell<Option<Transformation>>,
    /// Snapshot of the at-rest decision made the last time the
    /// per-frame transform was reconsidered (in `update` for `Layer`
    /// mode, in `draw` for `Bake` mode). Read by both `update` (when
    /// writing the slot) and `draw` (when sizing the cache texture)
    /// so they don't disagree on supersample / snap.
    at_rest_snapshot: Cell<bool>,
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
    compositing_mode: CompositingMode,
    auto_invalidate: bool,
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
            compositing_mode: CompositingMode::default(),
            auto_invalidate: true,
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

    /// Sets the [`CompositingMode`].
    ///
    /// Defaults to [`CompositingMode::Layer`]; pass [`CompositingMode::Bake`]
    /// to preserve pre-v8 source-order semantics at the cost of correct
    /// nested-animation compositing.
    pub fn compositing_mode(mut self, mode: CompositingMode) -> Self {
        self.compositing_mode = mode;
        self
    }

    /// Whether the cache should auto-invalidate when content reacts to an
    /// input event.
    ///
    /// Default: `true`. On every non-[`RedrawRequested`] event the widget
    /// snapshots the [`Shell`]'s state (redraw request, event-capture status,
    /// layout/widgets-invalid flags) before passing the event to its content,
    /// then compares afterwards. Any change is treated as evidence the
    /// content's visual state changed (e.g. a [`text_input`] absorbed a
    /// keystroke, a [`button`] entered a hover state, focus moved) and
    /// [`TextureCache::invalidate`] is called so the next draw re-records
    /// the texture.
    ///
    /// Disable when the cache is intentionally a frozen snapshot — e.g. an
    /// off-screen preview that should keep showing its original content even
    /// while interactions happen inside the cached subtree. With this off,
    /// the caller is responsible for calling
    /// [`TextureCache::invalidate`] from `update()` whenever they want the
    /// next frame to re-record.
    ///
    /// [`RedrawRequested`]: iced::window::Event::RedrawRequested
    /// [`text_input`]: iced::widget::text_input
    /// [`button`]: iced::widget::button
    pub fn auto_invalidate(mut self, enabled: bool) -> Self {
        self.auto_invalidate = enabled;
        self
    }
}

/// Subset of [`Shell`] state captured before passing an event to the
/// cached content. After the content's `update` returns, the same fields
/// are sampled again and any difference is taken as evidence the content
/// reacted and the cache should be invalidated — see the use site in
/// [`Cached::update`] for the rationale.
struct ShellSnapshot {
    redraw_request: window::RedrawRequest,
    event_status: iced::event::Status,
    is_layout_invalid: bool,
    are_widgets_invalid: bool,
}

/// Returns `true` if `t` is a pure 2D translation (identity linear part), the
/// case in which snapping the origin to the device grid is lossless. Reading
/// the raw matrix keeps this correct even if `Transformation` later gains a
/// `rotate` (rotation/shear leaves the off-diagonal terms non-zero).
fn is_translation_only(t: &Transformation) -> bool {
    let m: &[f32; 16] = t.as_ref();
    const EPS: f32 = 1e-4;
    (m[0] - 1.0).abs() < EPS && (m[5] - 1.0).abs() < EPS && m[1].abs() < EPS && m[4].abs() < EPS
}

/// Pre-computed geometry for one frame's cache record + composite.
///
/// Returned by [`Cached::composite_geometry`] and consumed by both
/// `update` (writing the slot) and `draw` (sizing the
/// `draw_to_texture` call), so the two phases never disagree on
/// supersample, snap, or padded size.
#[derive(Debug, Clone, Copy)]
struct CompositeGeom {
    /// Padded size to pass to `draw_to_texture`, in logical pixels.
    padded: Size<u32>,
    /// Texture-recording scale (`device_scale * supersample`).
    tex_scale: f32,
    /// Where the cache (padded) is composited, in logical coords.
    cache_bounds: Rectangle,
    /// Transform to apply during compositing. Includes the snap fix-up
    /// when `Auto` snapping decided to snap this frame.
    transform: Transformation,
}

impl<'a, Message, Theme, Renderer> Cached<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Computes the per-frame composite geometry given the widget's
    /// layout bounds, the device scale, and whether the animation is
    /// currently at rest.
    fn composite_geometry(&self, bounds: Rectangle, scale: f32, at_rest: bool) -> CompositeGeom {
        // Choose the record resolution. `auto_supersample_on_motion`
        // lifts it to at least 1.5x while moving (sharper
        // fractional-offset resampling) and drops back to the base
        // factor at rest, where we snap and want a 1:1 crisp blit.
        let base_ss = self.supersample.max(1.0);
        let ss = if self.auto_supersample_on_motion && !at_rest {
            base_ss.max(1.5)
        } else {
            base_ss
        };
        let tex_scale = scale * ss;

        // Reserve a transparent margin so the content's hard edges sit
        // *inside* the texture rather than on the texel boundary.
        // Without it, the sampler's ClampToEdge can't fade the kernel
        // into transparency and the edge "crawls" at a fractional
        // offset.
        let size = Size::new(
            bounds.width.ceil().max(1.0) as u32,
            bounds.height.ceil().max(1.0) as u32,
        );
        let padded = Size::new(size.width + 2 * MARGIN, size.height + 2 * MARGIN);

        // Composite the (padded) cache. Deriving the destination size
        // from the physical backing avoids the sub-pixel scale drift
        // that resamples and blurs at a fractional layout. The origin
        // is shifted back by MARGIN so the content lands exactly where
        // it would without the margin.
        let physical = Size::new(
            (padded.width as f32 * tex_scale).round(),
            (padded.height as f32 * tex_scale).round(),
        );

        // `Hybrid` snaps only the *bounds* portion of the composite
        // origin to the device-pixel grid. The user's transform stays
        // fractional (applied verbatim further down), so smooth
        // translate animations remain smooth while a discrete bounds
        // change — e.g. a layout reshuffle during a window resize —
        // no longer contributes to the bicubic-resampling phase, and
        // the perceived blur level stops "breathing" between frames.
        // Cost: visible position can shift by up to ½ device pixel
        // when `bounds.x` crosses a pixel boundary.
        let snap_bounds_only = matches!(self.pixel_snap, PixelSnap::Hybrid);
        let bx = if snap_bounds_only {
            (bounds.x * scale).round() / scale
        } else {
            bounds.x
        };
        let by = if snap_bounds_only {
            (bounds.y * scale).round() / scale
        } else {
            bounds.y
        };

        let cache_bounds = Rectangle {
            x: bx - MARGIN as f32,
            y: by - MARGIN as f32,
            width: physical.width / tex_scale,
            height: physical.height / tex_scale,
        };

        // Decide whether to snap the *full* composite. `Auto` snaps
        // only a pure translation that is **at rest**; any scale never
        // snaps. `Hybrid` is handled above via the bounds pre-snap,
        // so it falls into the no-extra-snap branch and the user's
        // `self.transform` is applied verbatim.
        let snap = match self.pixel_snap {
            PixelSnap::Always => true,
            PixelSnap::Never | PixelSnap::Hybrid => false,
            PixelSnap::Auto => is_translation_only(&self.transform) && at_rest,
        };
        let transform = if snap {
            // Snap the *texture/quad origin* (the cache_bounds origin),
            // not the content origin, so the whole texel grid lands on
            // the device grid and the resting frame samples at integer
            // phase, where Catmull-Rom is exact.
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

        CompositeGeom {
            padded,
            tex_scale,
            cache_bounds,
            transform,
        }
    }
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
        tree::State::new(State {
            slot: LayerSlot::new(self.cache.clone()),
            last_transform: Cell::new(None),
            at_rest_snapshot: Cell::new(true),
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        // Re-key the slot when the caller swaps in a different
        // `TextureCache` at the same tree position. The previous
        // slot's `Arc` clones in any frame-N registry simply expire on
        // the next `registry.clear()`.
        if state.slot.cache.id() != self.cache.id() {
            state.slot = LayerSlot::new(self.cache.clone());
            state.last_transform.set(None);
            state.at_rest_snapshot.set(true);
        }
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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let scale = renderer.scale_factor().unwrap_or(1.0);

        let layer_mode = self.compositing_mode == CompositingMode::Layer;
        let at_rest = if layer_mode {
            // Layer mode: `update` already decided this frame's
            // `at_rest` and wrote it to the snapshot before the draw
            // walk started. Reading the snapshot keeps `draw`'s
            // `composite_geometry` consistent with the slot the
            // compositor will use.
            state.at_rest_snapshot.get()
        } else {
            // Bake mode keeps pre-v8 semantics: draw both reads and
            // writes `last_transform`, so it computes `at_rest` here.
            let v = state
                .last_transform
                .get()
                .map_or(true, |prev| prev == self.transform);
            state.last_transform.set(Some(self.transform));
            state.at_rest_snapshot.set(v);
            v
        };

        let geom = self.composite_geometry(bounds, scale, at_rest);

        // Record the content into the cache. If the cache is fresh and
        // the size/scale match, the closure is skipped entirely by the
        // backend. In `Layer` mode the renderer's `recording_stack`
        // rises while the closure runs; any nested `Cached` inside the
        // closure will *still* register its own slot in `update`, and
        // its inline composite is suppressed in `Layer` mode — the
        // compose pass handles it.
        let content = &self.content;
        let content_tree = &tree.children[0];

        let record_self = self.cache.is_invalidated();
        let subtree_dirty = state.slot.take_subtree_dirty();

        let record_mode = if layer_mode {
            match record_self {
                true => TextureRecordMode::Flush,
                false => match subtree_dirty {
                    true => TextureRecordMode::TraverseOnly,
                    false => TextureRecordMode::Flush,
                },
            }
        } else {
            TextureRecordMode::Flush
        };

        // let record_mode = match record_self {
        //     true => TextureRecordMode::Flush,
        //     false => TextureRecordMode::TraverseOnly,
        // };

        let translated_cursor = translate_cursor(cursor, geom.transform);

        renderer.draw_to_texture(record_mode, &self.cache, geom.padded, geom.tex_scale, |r| {
            r.with_translation(
                Vector::new(-bounds.x + MARGIN as f32, -bounds.y + MARGIN as f32),
                |r| {
                    content.as_widget().draw(
                        content_tree,
                        r,
                        theme,
                        style,
                        layout,
                        translated_cursor,
                        viewport,
                    );
                },
            );
        });

        if self.compositing_mode == CompositingMode::Bake {
            // Inline source-order composite, exactly as pre-v8.
            renderer.with_transformation(geom.transform, |r| {
                r.draw_cached_texture(&self.cache, geom.cache_bounds);
            });
        }
        // Layer mode: composite is emitted by `renderer.compose_layers`
        // after the widget-tree draw walk completes.
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
        let is_redraw = matches!(event, Event::Window(window::Event::RedrawRequested(_)));
        let layer_mode = self.compositing_mode == CompositingMode::Layer;

        if is_redraw && layer_mode {
            // Compute this frame's geometry once and write it into the
            // slot. `at_rest_snapshot` is captured for `draw` to read.
            let state = tree.state.downcast_mut::<State>();
            let at_rest = state
                .last_transform
                .get()
                .map_or(true, |p| p == self.transform);
            state.last_transform.set(Some(self.transform));
            state.at_rest_snapshot.set(at_rest);

            let scale = renderer.scale_factor().unwrap_or(1.0);
            let geom = self.composite_geometry(layout.bounds(), scale, at_rest);

            // Cache parent id outside the slot write closure so we
            // don't try to borrow `shell` from inside the mutex guard.
            let parent_id = shell.current_layer().map(|s| s.id());

            state.slot.write(|d| {
                d.transform = geom.transform;
                d.bounds = geom.cache_bounds;
                d.parent_id = parent_id;
                d.clip_bounds = None;
            });

            // O(depth) cache coupling: if our cache is invalidated,
            // every ancestor layer's `draw_to_texture` closure must
            // run too so our `draw` is called inside them and our
            // cache gets re-recorded. The plain `is_invalidated()`
            // read does not consume the flag — `take_invalidated()` in
            // `start_recording_texture` does the consume during draw.
            // todo rewrite comment
            if self.cache.is_invalidated() {
                for ancestor in shell.layer_stack_ancestors() {
                    ancestor.mark_subtree_dirty();
                }
            }

            // Order matters: register THEN push, so the slot we just
            // wrote is also visible to nested layer widgets as their
            // `current_layer` parent (fix for v6 #1).
            shell.register_layer(state.slot.clone());
            shell.push_layer(&state.slot);
        }

        // Snapshot `shell` so we can tell if the content reacted to a
        // non-redraw event (typing, hover, click, focus, …). A reaction
        // means the cache texture is now stale and should be re-recorded.
        // Skip the check on `RedrawRequested` — animations that schedule
        // their own next frame would otherwise invalidate every frame and
        // defeat the cache.
        let snapshot = (self.auto_invalidate && !is_redraw).then(|| ShellSnapshot {
            redraw_request: shell.redraw_request(),
            event_status: shell.event_status(),
            is_layout_invalid: shell.is_layout_invalid(),
            are_widgets_invalid: shell.are_widgets_invalid(),
        });

        let translated_cursor =
            if let Some(transformation) = tree.state.downcast_ref::<State>().last_transform.get() {
                translate_cursor(cursor, transformation)
            } else {
                cursor
            };

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            translated_cursor,
            renderer,
            shell,
            viewport,
        );

        if let Some(prev) = snapshot {
            let reacted = shell.redraw_request() != prev.redraw_request
                || shell.event_status() != prev.event_status
                || shell.is_layout_invalid() != prev.is_layout_invalid
                || shell.are_widgets_invalid() != prev.are_widgets_invalid;
            if reacted {
                self.cache.invalidate();
            }
        }

        if is_redraw && layer_mode {
            shell.pop_layer();
        }
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
        let translated_cursor = if let Some(transformation) = state.last_transform.get() {
            translate_cursor(cursor, transformation)
        } else {
            cursor
        };

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            translated_cursor,
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
        // Pure pass-through — focus/scroll/text-input traverse the
        // widget tree unmodified.
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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

#[inline]
fn translate_cursor(cursor: mouse::Cursor, transform: Transformation) -> mouse::Cursor {
    let t = transform.translation();
    let s = transform.scale_factor();

    let map_point = |p: Point| Point::new((p.x - t.x) / s, (p.y - t.y) / s);

    match cursor {
        mouse::Cursor::Available(p) => mouse::Cursor::Available(map_point(p)),
        mouse::Cursor::Unavailable => mouse::Cursor::Unavailable,
        mouse::Cursor::Levitating(p) => mouse::Cursor::Levitating(map_point(p)),
    }
}
