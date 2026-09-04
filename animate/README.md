# `iced_animate`

Tree-external animation engine for [iced](https://iced.rs) 0.14: keyed
springs and eases that widgets resolve inside their own `layout` and `draw`,
so nothing publishes a message per frame.

## Minimal example

```rust
use iced::widget::column;
use iced::{Color, Element};
use iced_animate::widget::shape;
use iced_animate::{curves::SMOOTH, key, Motion};

struct App {
    motion: Motion,
    open: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => self.open = !self.open,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Idempotent: an unchanged target is a touch, a changed one retargets
        // the spring without discarding its velocity.
        let width = self
            .motion
            .to(key!(), SMOOTH, if self.open { 240.0 } else { 40.0 });
        let fill = self.motion.to(
            key!(),
            SMOOTH,
            if self.open { Color::WHITE } else { Color::BLACK },
        );

        // The host is the clock: wrap the root of the view in it.
        self.motion
            .host(column![shape().width(width).height(40.0).fill(fill)])
            .into()
    }
}
```

Add to `Cargo.toml`:

```toml
[dependencies]
iced = "0.14"
iced_animate = "0.1"
```

## Concepts

### One rule

Read animated values inside a widget, not while building the view. `view()`
runs when application state changes, such as after a click. It does not run
for every frame. A value read there is a snapshot that stays fixed until the
next rebuild.
So `Motion` hands out `Anim<T>` handles instead of numbers, and the widgets in
this crate take `Anim<T>` (or `AnimLength`) rather than `f32`.

### Keys

Animation state lives outside the widget tree, addressed by a `MotionKey`.
`key!()` derives one from the call site; `key!(id)`, `key!("width")` add
discriminators so one call site can own several tracks. `MotionKey::unique()`
mints a fresh key for widget-owned state; `MotionKey::with(&value)` and
`salted(u64)` derive keys by hand. Tree state is positional and disappears
when an element leaves the view; a keyed track survives both, which is what
makes exit animations possible.

### Vocabulary

| Call | Meaning |
|---|---|
| `to(key, curve, target)` | Animate towards `target`; idempotent per rebuild. |
| `to_set(key, curve, set)` | Same for a `motion_set!` struct: every field gets its own track under one curve. |
| `play(key, curve, from, to)` | One-shot: restart from `from` every time it is called. |
| `enter(key, curve, from, to)` | Like `play`, but replays only the first time the key is seen. |
| `retire(key, curve, to)` | Start the exit animation; `presence(key)` reports `Exiting` then `Gone`. |
| `get::<T>(key)` | Look a track up without retargeting it. |
| `host(content)` | Wrap the root of the view; ticks the engine on every redraw. |

`Motion::end_build()` and `Motion::collect()` are for hosts that are not the
provided widget: the first closes a build (tracks not touched since become
collectable), the second drops tracks nobody holds any more.
`Motion::track_count()` is for tests.

### Tiers

Every track has a `Tier` set by the widget that binds it: `Composite`
(redraw without recording cached textures again, used by
`iced_texture_cache::Cached`), `Paint` (redraw), `Layout` (relayout and
redraw). The host invalidates exactly what the moving tracks require. A track
no widget has marked yet is treated as `Paint`.

### Curves

`curves::SMOOTH`, `QUICK`, `BOUNCY`, and `STRUCTURAL` are springs. They keep
velocity across retargets and use a closed-form calculation, so any frame
interval is exact. `FADE` and `COLLAPSE` are eases. Build your own with
`Curve::spring(SpringParams::new(bounce, duration))` or
`Curve::ease(Easing::EaseInOut, duration)`; `.delayed(duration)` sets the
delay before the curve runs.

### Widgets

`widget::Shape` (`shape()`): a rectangle whose fill, radius and border are
animated; resolved in `draw`. `widget::Sized` (`sized(content)`): a box whose
width, height, padding and collapse are animated; resolved in `layout`.
`widget::Host` (`host(motion, content)`): the clock. All three are generic over
iced's `Theme` and `Renderer`.

## Feature flags

None. The crate depends on `iced_core` and `log` only.

## Limitations

* Values are interpolated per component (`Animatable`, at most
  `MAX_COMPONENTS` = 4); there is no path or keyframe animation.
* Non-finite targets are replaced by the current value and logged as an error.
* A track is advanced only while a `Host` is in the view; without one, nothing
  moves. One `Motion` per window: two hosts ticking one engine in the same
  build is reported once and the second host is ignored for timing.

## Related crates

* [`iced_texture_cache`](https://crates.io/crates/iced_texture_cache): the
  compositor tier: `Cached` moves, scales and fades a recorded texture.
* [`iced_page_router`](https://crates.io/crates/iced_page_router): pages and
  history, independent of this crate.
* [`iced_luminate`](https://crates.io/crates/iced_luminate): a design kit built on
  all of the above.

Examples: `cargo run -p iced_animate --example tiers` (`ANIM_AUTOPLAY=1` flips
the demos every 1.5 s). The examples live in the
[repository](https://github.com/sonata-ltd/luminate/tree/master/animate/examples)
and are not part of the published crate. Changes are listed in the workspace
[CHANGELOG](https://github.com/sonata-ltd/luminate/blob/master/CHANGELOG.md).
