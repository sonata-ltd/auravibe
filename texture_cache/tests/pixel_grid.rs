//! A shape grown by an animated ancestor must not step by whole pixels.
//!
//! iced's `crisp` feature snaps a quad's *both* edges to the device grid, so a
//! rectangle animated through the layout changes size in whole-pixel lurches
//! unless the widget turns snapping off while it moves. `iced_animate::shape`
//! does, and this is the end-to-end proof.
//!
//! It lives in this crate because the assertion needs a GPU: the snap is
//! applied by `iced_wgpu`'s quad shader, and the software backend ignores the
//! flag entirely.
#![cfg(feature = "wgpu")]

use std::time::Duration;

use iced_animate::widget::{shape, sized};
use iced_animate::{Curve, Motion, SpringParams, key};
use iced_core::Renderer as _;
use iced_core::renderer::{Headless, Quad};
use iced_core::{Color, Element, Font, Pixels, Rectangle, Size, mouse, renderer, window};
use iced_core::{Event, clipboard};
use iced_test::runtime::user_interface::{self, UserInterface};
use iced_texture_cache::Renderer;

const CANVAS: Size = Size::new(200.0, 200.0);
const FRAME: Duration = Duration::from_millis(16);
const FROM: f32 = 40.0;
const TO: f32 = 76.0;

/// A fast spring, so the growth is over in a dozen frames.
const FAST: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(300)));

/// The side of the drawn square, in device pixels.
///
/// The screenshot is a black square on white, and anti-aliased coverage sums
/// to the true area, so the square root of the ink is the side — fractions
/// included. That is exactly what a snapped quad cannot produce.
fn side_of(rgba: &[u8]) -> f64 {
    let ink = rgba
        .chunks_exact(4)
        .map(|p| (255.0 - f64::from(p[0])) / 255.0)
        .sum::<f64>();

    ink.sqrt()
}

struct Harness {
    renderer: Renderer,
    motion: Motion,
    cache: Option<user_interface::Cache>,
    now: iced_core::time::Instant,
}

impl Harness {
    fn new() -> Self {
        let renderer = iced_test::futures::futures::executor::block_on(
            <Renderer as Headless>::new(Font::DEFAULT, Pixels(16.0), Some("wgpu")),
        )
        .expect("a GPU adapter is available");

        Self {
            renderer,
            motion: Motion::new(),
            cache: Some(user_interface::Cache::default()),
            now: iced_core::time::Instant::now(),
        }
    }

    /// One frame of the `size (layout)` demo: an animated `Sized` around a
    /// shape whose own values are all constants.
    fn frame(&mut self, target: f32) -> f64 {
        self.now += FRAME;

        let side = self.motion.to(key!(), FAST, target);
        let square: Element<'_, (), iced::Theme, Renderer> = sized(shape().fill(Color::BLACK))
            .width(side.clone())
            .height(side)
            .into();

        let cache = self.cache.take().expect("returned after every frame");
        let mut messages = Vec::new();
        let mut ui: UserInterface<'_, (), iced::Theme, Renderer> =
            UserInterface::build(self.motion.host(square), CANVAS, cache, &mut self.renderer);
        let _ = ui.update(
            &[Event::Window(window::Event::RedrawRequested(self.now))],
            mouse::Cursor::Unavailable,
            &mut self.renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        self.renderer.reset(Rectangle::with_size(CANVAS));
        ui.draw(
            &mut self.renderer,
            &iced::Theme::Light,
            &renderer::Style {
                text_color: Color::BLACK,
            },
            mouse::Cursor::Unavailable,
        );
        let shot = self.renderer.screenshot(
            Size::new(CANVAS.width as u32, CANVAS.height as u32),
            1.0,
            Color::WHITE,
        );
        self.cache = Some(ui.into_cache());

        side_of(&shot)
    }
}

#[test]
#[ignore = "needs a GPU adapter"]
fn a_shape_grown_through_the_layout_lands_between_pixels() {
    if !Quad::default().snap {
        println!("`crisp` is off: nothing snaps, nothing to prove");
        return;
    }

    let mut harness = Harness::new();
    for _ in 0..3 {
        let side = harness.frame(FROM);
        assert!(
            (side - f64::from(FROM)).abs() < 0.5,
            "at rest the square is {FROM} px, measured {side}"
        );
    }

    let sides: Vec<f64> = (0..14).map(|_| harness.frame(TO)).collect();

    assert!(
        sides.windows(2).all(|w| w[1] >= w[0] - 0.01),
        "the growth is monotonic: {sides:?}"
    );
    assert!(
        sides
            .last()
            .is_some_and(|side| *side > f64::from(FROM) + 20.0),
        "the square really did grow: {sides:?}"
    );

    // The point of the test. Snapped, every one of these is a whole number of
    // pixels; unsnapped, most land between two.
    let fractional = sides
        .iter()
        .filter(|side| (*side - side.round()).abs() > 0.05)
        .count();
    assert!(
        fractional * 2 > sides.len(),
        "only {fractional} of {} frames drew a fractional side; the quad is \
         being snapped while it moves: {sides:?}",
        sides.len()
    );
}
