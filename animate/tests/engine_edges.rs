//! Small promises of the public surface: delays under retarget, custom
//! easings, degenerate durations, stalls, and the `key!` generic trap.

use std::any::TypeId;
use std::time::Duration;

use iced::time::Instant;
use iced::{Length, Padding, Pixels};
use iced_animate::testing::FrameClock;
use iced_animate::{
    Anim, AnimLength, Curve, Easing, Motion, MotionKey, Presence, SpringParams, Tier, key,
    motion_set,
};

const FAST: Curve = Curve::spring(SpringParams::new(0.0, Duration::from_millis(300)));

fn linear(t: f32) -> f32 {
    t
}

fn square(t: f32) -> f32 {
    t * t
}

#[test]
fn a_delayed_spring_holds_through_its_delay_and_then_moves_to_the_latest_target() {
    let m = Motion::new();
    let k = key!();
    let delayed = FAST.delayed(Duration::from_millis(100));
    let _ = m.to(k, delayed, 0.0_f32);
    let mut clock = FrameClock::new(&m);
    let _ = clock.run(1); // starts the clock

    let first = m.to(k, delayed, 100.0_f32);
    let _ = clock.run(3); // ~50 ms: still inside the delay
    assert_eq!(first.get(), 0.0, "the pose is held during the delay");

    let second = m.to(k, delayed, 50.0_f32);
    let _ = clock.run(2); // ~83 ms: still held
    assert_eq!(second.get(), 0.0);

    let frames = clock.run_until_settled();
    assert!(frames > 0);
    assert!(
        (second.get() - 50.0).abs() < 0.01,
        "moved to the latest target: {}",
        second.get()
    );
}

#[test]
fn custom_easings_compare_by_function_pointer() {
    let duration = Duration::from_millis(200);
    assert_eq!(
        Curve::ease(Easing::Custom(linear), duration).kind(),
        Curve::ease(Easing::Custom(linear), duration).kind()
    );
    assert_ne!(
        Curve::ease(Easing::Custom(linear), duration).kind(),
        Curve::ease(Easing::Custom(square), duration).kind()
    );

    // Same curve, same target: retargeting is a no-op, so the value keeps moving.
    let motion = Motion::new();
    let key = key!();
    let curve = Curve::ease(Easing::Custom(linear), duration);
    let _ = motion.to(key, curve, 0.0_f32);
    let mut clock = FrameClock::new(&motion);
    let first = motion.to(key, curve, 10.0_f32);
    let _ = clock.run(6);
    let midway = first.get();
    assert!(midway > 0.0 && midway < 10.0, "in flight: {midway}");
    let again = motion.to(key, curve, 10.0_f32);
    assert_eq!(
        again.get(),
        midway,
        "re-declaring the same target does not restart"
    );
}

#[test]
fn a_zero_duration_ease_settles_on_its_first_frame() {
    let m = Motion::new();
    let k = key!();
    let instant = Curve::ease(Easing::Linear, Duration::ZERO);
    let _ = m.to(k, instant, 0.0_f32);
    let mut clock = FrameClock::new(&m);
    let _ = clock.run(1); // starts the clock
    let a = m.to(k, instant, 10.0_f32);
    // The engine was at rest, so the first frame after the retarget restarts
    // the clock; the second is the first one with time to spend.
    let _ = clock.run(2);
    assert_eq!(a.get(), 10.0);
    assert!(!a.is_animating());
    assert!(!clock.run(1).animating, "nothing left to drive");
}

#[test]
fn an_animation_started_after_an_idle_stretch_plays_in_full() {
    // Frames are drawn on demand, so an interface at rest draws none at all.
    // The click that starts an animation therefore arrives an arbitrarily
    // long time after the last frame, and that interval is not animation
    // time: spending it would play the whole spring in one step.
    let m = Motion::new();
    let k = key!();
    let _ = m.to(k, FAST, 0.0_f32);
    let start = Instant::now();
    let _ = m.tick(start);

    let a = m.to(k, FAST, 10.0_f32);
    let mut at = start + Duration::from_secs(5);
    let _ = m.tick(at);
    assert_eq!(
        a.get(),
        0.0,
        "the frame that restarts the clock only starts it"
    );

    at += Duration::from_millis(16);
    let _ = m.tick(at);
    let first = a.get();
    assert!(
        first > 0.0 && first < 3.0,
        "one frame of a 300 ms spring, not five seconds of it: {first}"
    );

    let mut frames = 1;
    while frames < 600 {
        at += Duration::from_millis(16);
        frames += 1;

        if !m.tick(at).animating {
            break;
        }
    }
    assert!(frames > 5, "it played out over many frames: {frames}");
    assert!(
        (a.get() - 10.0).abs() < 1e-3,
        "and still arrives: {}",
        a.get()
    );
}

#[test]
fn retiring_an_unseen_key_is_gone_at_once() {
    let m = Motion::new();
    let k = key!();
    let a = m.retire(k, FAST, 0.0_f32);
    assert_eq!(m.presence(k), Presence::Gone);
    assert!(
        !a.is_animating(),
        "created at its exit pose, already settled"
    );
    let mut clock = FrameClock::new(&m);
    assert!(!clock.run(2).animating, "nothing to animate out");
}

#[test]
fn a_default_anim_is_a_still_constant() {
    let a = Anim::<f32>::default();
    assert_eq!(a.get(), 0.0);
    assert!(!a.is_live());
    assert_eq!(a.tier(), None);
}

#[test]
fn lengths_convert_from_pixels_and_lengths() {
    assert_eq!(AnimLength::from(Pixels(4.0)).resolve(), Length::Fixed(4.0));
    assert_eq!(AnimLength::from(Length::Fill).resolve(), Length::Fill);
    assert_eq!(AnimLength::from(3.5_f32).resolve(), Length::Fixed(3.5));
    assert_eq!(AnimLength::from(7_u32).resolve(), Length::Fixed(7.0));
}

motion_set! {
    /// A two-field set used only by `a_motion_set_twin_is_default`.
    struct Row -> RowAnim {
        /// Text size.
        size: f32,
        /// Padding around the row.
        pad: Padding,
    }
}

#[test]
fn a_motion_set_twin_is_default() {
    let twin = RowAnim::default();
    assert_eq!(twin.size.get(), 0.0);
    assert_eq!(twin.pad.get(), Padding::ZERO);
    assert!(!twin.size.is_live());
}

#[test]
fn a_layout_track_stops_invalidating_once_settled() {
    let m = Motion::new();
    let k = key!();
    let _ = m.to(k, FAST, 0.0_f32);
    let mut clock = FrameClock::new(&m);
    let _ = clock.run(1); // starts the clock
    let a = m.to(k, FAST, 10.0_f32);
    a.mark_tier(Tier::Layout);
    // One frame to restart the clock after the idle stretch, one to move.
    let moving = clock.run(2);
    assert!(moving.animating && moving.layout_invalid);
    let _ = clock.run_until_settled();
    let settled = clock.run(1);
    assert!(!settled.animating);
    assert!(
        !settled.layout_invalid,
        "a settled layout track must not keep the layout dirty"
    );
}

/// A generic widget that keys its track from one call site.
struct Lane<T>(std::marker::PhantomData<T>);

impl<T: 'static> Lane<T> {
    fn plain_key() -> MotionKey {
        key!()
    }

    fn typed_key() -> MotionKey {
        key!(TypeId::of::<T>())
    }
}

#[test]
fn a_generic_site_collides_across_monomorphisations_unless_it_names_the_type() {
    assert_eq!(
        Lane::<u8>::plain_key(),
        Lane::<u16>::plain_key(),
        "same site, same key: the documented trap"
    );
    assert_ne!(
        Lane::<u8>::typed_key(),
        Lane::<u16>::typed_key(),
        "adding the TypeId separates them"
    );
}
