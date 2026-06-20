use iced::{Rectangle, Size, Transformation};

/// Pre-computed geometry for one frame's cache record + composite.
///
/// Returned by [`Cached::composite_geometry`] and consumed by both
/// `update` (writing the slot) and `draw` (sizing the
/// `draw_to_texture` call), so the two phases never disagree on
/// supersample, snap, or padded size.
#[derive(Debug, Clone, Copy)]
pub struct CompositeGeom {
    /// Padded size to pass to `draw_to_texture`, in logical pixels.
    pub padded: Size<u32>,
    /// Texture-recording scale (`device_scale * supersample`).
    pub tex_scale: f32,
    /// Where the cache (padded) is composited, in logical coords.
    pub cache_bounds: Rectangle,
    /// Transform to apply during compositing. Includes the snap fix-up
    /// when `Auto` snapping decided to snap this frame.
    pub transform: Transformation,
}

/// Computes the per-frame composite geometry given the widget's
/// layout bounds, the device scale, and whether the animation is
/// currently at rest.
pub fn composite_geometry(
    texture_margin: u32,
    bounds: Rectangle,
    scale: f32,
    transform: Transformation,
) -> CompositeGeom {
    let size = Size::new(
        bounds.width.ceil().max(1.0) as u32,
        bounds.height.ceil().max(1.0) as u32,
    );
    let padded = Size::new(
        size.width + 2 * texture_margin,
        size.height + 2 * texture_margin,
    );
    let tex_scale = scale; // supersample = 1.0

    let physical = Size::new(
        (padded.width as f32 * tex_scale).round(),
        (padded.height as f32 * tex_scale).round(),
    );

    let bx = (bounds.x * scale).round() / scale;
    let by = (bounds.y * scale).round() / scale;

    let cache_bounds = Rectangle {
        x: bx - texture_margin as f32,
        y: by - texture_margin as f32,
        width: physical.width / tex_scale,
        height: physical.height / tex_scale,
    };

    CompositeGeom {
        padded,
        tex_scale,
        cache_bounds,
        transform,
    }
}
