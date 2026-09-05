//! Shared animation curve presets.
//!
//! Declared in one place so that everything built on the engine moves with
//! one voice, and retuning the feel of an interface is an edit in one file.
//!
//! Spring durations here are *perceptual* durations, calibrated exactly as
//! `SwiftUI`'s `Spring(duration:bounce:)` is (see [`SpringParams::new`]), so
//! these presets and Apple's are directly comparable — and this set runs a
//! little shorter than Apple's own, which are tuned for touch, where a finger
//! has already told the eye where things are going. A pointer-driven desktop
//! interface reads the same motion as unhurried at these lengths. Apple's
//! numbers (`.smooth` 0.5 s, `.snappy` 0.5 s / bounce 0.15, `.bouncy` 0.5 s /
//! bounce 0.3) can be typed in directly wherever they suit better.
//!
//! [`sharp`] is the same four springs about 1.6× brisker again.

use std::time::Duration;

use crate::Easing;

use crate::{Curve, SpringParams};

/// Parameters behind [`SMOOTH`]; also [`SpringParams::default`].
pub(crate) const SMOOTH_PARAMS: SpringParams = SpringParams::new(0.0, Duration::from_millis(400));

/// The default transition: settles quickly, does not overshoot.
///
/// Use this unless there is a reason not to. It is a spring, so a value
/// retargeted mid-flight continues from its current motion. Apple's
/// `Spring.smooth` with a fifth off the duration: 99 % of the way there in
/// 430 ms rather than 535 ms.
pub const SMOOTH: Curve = Curve::spring(SMOOTH_PARAMS);

/// A brisker [`SMOOTH`], for small elements that should feel immediate.
///
/// Between Apple's `.smooth` and its `.interactiveSpring`: short enough that
/// a control answers a click without ceremony, long enough to still read as
/// movement rather than a cut.
pub const QUICK: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(220)));

/// Overshoots slightly on arrival, for something appearing, not adjusting.
///
/// Apple's `Spring.bouncy` with a touch more bounce: a 7 % overshoot, which
/// is plenty. Keep it for small elements — a chip, a badge, a toggle.
/// Overshoot on something large,
/// and especially on something carrying text through
/// [`Cached`](https://docs.rs/iced_texture_cache), spends extra frames off
/// the pixel grid, where the compositor has to resample and the text softens.
pub const BOUNCY: Curve = Curve::spring(SpringParams::new(0.35, Duration::from_millis(500)));

/// Structural motion: a panel collapsing, a stack sliding between pages.
///
/// Slower and heavier than [`SMOOTH`] because it moves a large area, where a
/// fast transition reads as a jump rather than a movement. No bounce, for the
/// reason given on [`BOUNCY`].
pub const STRUCTURAL: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(450)));

/// How long [`FADE`] takes.
///
/// Named so [`COLLAPSE`] can wait exactly that long rather than repeating a
/// number that would drift out of step the first time the fade is retuned.
const FADE_MS: u64 = 180;

/// Closing the space an element leaves behind, once it has faded out.
///
/// Delayed by the full length of [`FADE`], on purpose. A collapse clips what
/// it shrinks, so overlapping the two cuts the element in half against
/// whatever lies below while it is still visible. Waiting means the clip only
/// ever eats something already invisible.
pub const COLLAPSE: Curve = Curve::ease(Easing::EaseInOut, Duration::from_millis(220))
    .delayed(Duration::from_millis(FADE_MS));

/// A plain fade, where spring physics would be meaningless.
///
/// Opacity has no momentum to preserve, so a duration curve is both cheaper
/// and more predictable than a spring.
pub const FADE: Curve = Curve::ease(Easing::EaseOut, Duration::from_millis(FADE_MS));

/// The pace this kit moved at before its springs were calibrated against
/// `SwiftUI`'s: the same four springs, `2π / 10` as long, so about 1.6× brisker.
///
/// Not a lesser set. Apple's durations are tuned for touch, where a finger
/// has already told the eye where things are going; a pointer-driven desktop
/// interface, and especially a dense one, can carry motion this fast without
/// reading as abrupt. Reach for these when a whole interface feels ponderous
/// rather than calm — and reach for them together, since a set half in one
/// pace and half in the other is what actually reads as inconsistent.
///
/// The durations are given in microseconds to land on exactly the frequencies
/// the kit used before; whole milliseconds would be within 0.2 %, which is
/// imperceptible but would make the parity claim untrue.
///
/// ```
/// use iced_animate::curves::sharp;
/// use iced_animate::{Motion, key};
///
/// let motion = Motion::new();
/// let opacity = motion.to(key!(), sharp::SMOOTH, 1.0_f32);
/// # let _ = opacity;
/// ```
pub mod sharp {
    use std::time::Duration;

    use crate::{Curve, SpringParams};

    /// [`SMOOTH`](super::SMOOTH) at the kit's former pace.
    pub const SMOOTH: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_micros(251_327)));

    /// [`QUICK`](super::QUICK) at the kit's former pace. As brisk as Apple's
    /// `.interactiveSpring`, which is the fastest spring Apple ships.
    pub const QUICK: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_micros(138_230)));

    /// [`BOUNCY`](super::BOUNCY) at the kit's former pace.
    pub const BOUNCY: Curve =
        Curve::spring(SpringParams::new(0.35, Duration::from_micros(314_159)));

    /// [`STRUCTURAL`](super::STRUCTURAL) at the kit's former pace.
    pub const STRUCTURAL: Curve =
        Curve::spring(SpringParams::new(0.0, Duration::from_micros(282_743)));
}

#[cfg(test)]
mod tests {
    use super::{QUICK, SMOOTH, STRUCTURAL, sharp};
    use crate::testing::{FRAME, FrameClock};
    use crate::{Curve, Motion, key};

    /// Seconds until a track under `curve` covers 99 % of a 0 → 100 move.
    fn arrival(curve: Curve) -> f32 {
        let m = Motion::new();
        let k = key!();
        let mut clock = FrameClock::new(&m);

        let _ = m.to(k, curve, 0.0_f32);
        let _ = clock.run(1); // starts the clock
        let value = m.to(k, curve, 100.0_f32);
        let _ = clock.run(1); // restarts it after the idle stretch

        for frame in 1..10_000 {
            let _ = clock.run(1);

            if value.get() >= 99.0 {
                return frame as f32 * FRAME.as_secs_f32();
            }
        }

        f32::INFINITY
    }

    #[test]
    fn a_spring_arrives_within_its_perceptual_duration() {
        // The calibration in one assertion: a no-bounce spring is 99 % done
        // when its duration elapses. One 60 Hz frame is 6 % of the shortest
        // preset here, hence the 15 % bound.
        for (name, curve, duration) in [
            ("SMOOTH", SMOOTH, 0.4),
            ("QUICK", QUICK, 0.22),
            ("STRUCTURAL", STRUCTURAL, 0.45),
            ("sharp::SMOOTH", sharp::SMOOTH, 0.251_327),
            ("sharp::QUICK", sharp::QUICK, 0.138_230),
            ("sharp::STRUCTURAL", sharp::STRUCTURAL, 0.282_743),
        ] {
            let arrived = arrival(curve);
            assert!(
                (arrived - duration).abs() <= duration * 0.15,
                "{name}: duration {duration} s, arrived at {arrived} s"
            );
        }
    }

    #[test]
    fn the_sharp_set_is_the_pace_the_kit_shipped_before_the_calibration() {
        // Those springs were built as `ω = 10 / duration` from 400, 220 and
        // 450 ms, and a critically damped step response is 99 % done at
        // `6.638 / ω`. The `sharp` durations exist to reproduce exactly that.
        //
        // The numbers below coincide with today's durations, which is a
        // coincidence and not a shorthand: there they are perceptual
        // durations, here they are the settle-calibrated ones they replaced.
        for (name, curve, former) in [
            ("SMOOTH", sharp::SMOOTH, 0.4),
            ("QUICK", sharp::QUICK, 0.22),
            ("STRUCTURAL", sharp::STRUCTURAL, 0.45),
        ] {
            let expected = 6.638 / (10.0 / former);
            let arrived = arrival(curve);
            assert!(
                (arrived - expected).abs() <= 2.0 * FRAME.as_secs_f32(),
                "sharp::{name}: expected the former {expected} s, got {arrived} s"
            );
        }
    }

    #[test]
    fn the_sharp_set_moves_faster_than_the_shipped_one() {
        for (name, shipped, brisk) in [
            ("SMOOTH", SMOOTH, sharp::SMOOTH),
            ("QUICK", QUICK, sharp::QUICK),
            ("STRUCTURAL", STRUCTURAL, sharp::STRUCTURAL),
        ] {
            let (slow, fast) = (arrival(shipped), arrival(brisk));
            assert!(fast < slow, "{name}: {fast} s is not brisker than {slow} s");
        }
    }
}
