//! Demonstrates the `Cached` widget from `iced_auravibe`.
//!
//! Two stacked layers exercise the v8 compositor:
//!  - The outer card is rasterized into a `TextureCache` and slides
//!    ±100 logical pixels horizontally.
//!  - Inside the outer card, an inner `Cached` bobs ±20 logical pixels
//!    *vertically*. With `CompositingMode::Layer` (the default), both
//!    animate simultaneously without ghosting — the inner is composed
//!    on top of the outer with the parent slide transform applied,
//!    proving nested-animation correctness.
//!
//! Buttons:
//!  - `Reset animation` — resets the elapsed timer (the *transform*
//!    parameter) without invalidating either cache.
//!  - `Invalidate outer` — forces the outer card to re-record. The
//!    inner re-records too because the v8 O(depth) cache-coupling
//!    walks ancestor layers when our cache is invalidated.
//!  - `Invalidate inner` — forces the inner to re-record only. (The
//!    outer will still see its content via the layer composite pass.)
//!  - `Supersample`, `Snap`, `Motion SS` — knobs on both caches.
//!  - `Compositing` — switches between `Layer` (default, correct
//!    nested animation) and `Bake` (source-order, inner pose baked
//!    into outer texture).
use std::time::Instant;

use iced::{
    Border, Color, Element, Length, Settings, Subscription, Task, TextureCache, Theme,
    Transformation,
    widget::{button, column, container, row, text, text_input},
    window,
};
use iced_auravibe::cached::{Cached, CompositingMode, PixelSnap};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::sub)
        .title("Cached widget demo")
        .theme(App::theme)
        .run()
}

struct App {
    started_at: Option<Instant>,
    elapsed: f32,
    paused: bool,
    outer_cache: TextureCache,
    inner_cache: TextureCache,
    second_cache: TextureCache,
    third_cache: TextureCache,
    fourth_cache: TextureCache,
    supersample: f32,
    snap_mode: PixelSnap,
    auto_motion_ss: bool,
    compositing: CompositingMode,

    input_value: String,
}

#[derive(Clone, Debug)]
enum Message {
    Tick(Instant),
    Reset,
    TogglePause,
    InvalidateOuter,
    InvalidateInner,
    ToggleSupersample,
    ToggleSnap,
    ToggleMotionSS,
    ToggleCompositing,
    InputValueUpdate(String),
    EmptyButtonEmit,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                started_at: None,
                elapsed: 0.0,
                paused: false,
                outer_cache: TextureCache::new(),
                inner_cache: TextureCache::new(),
                second_cache: TextureCache::new(),
                third_cache: TextureCache::new(),
                fourth_cache: TextureCache::new(),
                // Default 1×: with PixelSnap::Auto, a pure-translate animation
                // stays crisp without supersampling — the browser-style path.
                supersample: 1.0,
                snap_mode: PixelSnap::Auto,
                auto_motion_ss: false,
                compositing: CompositingMode::Bake,
                input_value: String::new(),
            },
            Task::none(),
        )
    }

    fn sub(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Tick(now) => {
                if !self.paused {
                    let started_at = *self.started_at.get_or_insert(now);
                    self.elapsed = now.duration_since(started_at).as_secs_f32();
                } else {
                    // Re-anchor on the next un-pause so `elapsed` resumes
                    // from its current value rather than jumping by the
                    // paused duration.
                    self.started_at = None;
                }
            }
            Message::Reset => {
                self.started_at = None;
                self.elapsed = 0.0;
            }
            Message::TogglePause => {
                self.paused = !self.paused;
                if self.paused {
                    self.started_at = None;
                }
            }
            Message::InvalidateOuter => {
                self.outer_cache.invalidate();
            }
            Message::InvalidateInner => {
                self.inner_cache.invalidate();
            }
            Message::ToggleSupersample => {
                self.supersample = if self.supersample > 1.0 { 1.0 } else { 2.0 };
            }
            Message::ToggleSnap => {
                self.snap_mode = match self.snap_mode {
                    PixelSnap::Auto => PixelSnap::Always,
                    PixelSnap::Always => PixelSnap::Never,
                    PixelSnap::Never => PixelSnap::Hybrid,
                    PixelSnap::Hybrid => PixelSnap::Auto,
                };
            }
            Message::ToggleMotionSS => {
                self.auto_motion_ss = !self.auto_motion_ss;
            }
            Message::ToggleCompositing => {
                self.compositing = match self.compositing {
                    CompositingMode::Layer => CompositingMode::Bake,
                    CompositingMode::Bake => CompositingMode::Layer,
                };
                // Switching modes changes the composite pipeline; force
                // a re-record so both caches reflect the new layout
                // immediately.
                self.outer_cache.invalidate();
                self.inner_cache.invalidate();
                self.second_cache.invalidate();
                self.third_cache.invalidate();
                self.fourth_cache.invalidate();
            }
            Message::InputValueUpdate(s) => self.input_value = s,
            Message::EmptyButtonEmit => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Outer transform: ±100 logical px horizontal sinusoid.
        let outer_dx = (self.elapsed * 1.5).sin() * 100.0;
        let outer_transform = Transformation::translate(outer_dx, 0.0);

        // Inner transform: ±20 logical px vertical sinusoid, 2× faster.
        // Perpendicular to the outer so visual cross-talk between the
        // two would be obvious if the compositor got it wrong.
        // let inner_dy = (self.elapsed * 3.0).sin() * 20.0;
        let inner_transform = Transformation::translate(0.0, outer_dx);

        // The inner cached element — a small badge inside the card.
        let inner_inner = || {
            container(text(format!("inner@t={:.2}s", self.elapsed)).size(12))
                .padding(6)
                .style(|_t| container::Style {
                    background: Some(Color::from_rgb(1.0, 0.93, 0.78).into()),
                    text_color: Some(Color::BLACK),
                    border: Border {
                        color: Color::from_rgb(0.85, 0.55, 0.1),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .width(120)
                .height(25)
        };

        let inner_cached0_1 = Cached::new(self.inner_cache.clone(), inner_inner())
            .transform(inner_transform)
            .supersample(self.supersample)
            .pixel_snap(self.snap_mode)
            .auto_supersample_on_motion(self.auto_motion_ss)
            .compositing_mode(self.compositing);

        let inner_cached0_2 = Cached::new(self.second_cache.clone(), inner_inner())
            .transform(Transformation::IDENTITY)
            .supersample(self.supersample)
            .pixel_snap(self.snap_mode)
            .auto_supersample_on_motion(self.auto_motion_ss)
            .compositing_mode(self.compositing);

        let inner_cached1_1 = Cached::new(self.third_cache.clone(), inner_cached0_2)
            .transform(inner_transform)
            .supersample(self.supersample)
            .pixel_snap(self.snap_mode)
            .auto_supersample_on_motion(self.auto_motion_ss)
            .compositing_mode(self.compositing);

        let inner_cached0_3 = Cached::new(self.fourth_cache.clone(), inner_inner())
            .transform(inner_transform)
            .supersample(self.supersample)
            .pixel_snap(self.snap_mode)
            .auto_supersample_on_motion(self.auto_motion_ss)
            .compositing_mode(self.compositing);

        // The "expensive" outer content: a colored card with text,
        // buttons, and the nested cached element above.
        let card = container(
            column![
                button(text("inner button A"))
                    .padding(8)
                    .on_press(Message::EmptyButtonEmit),
                button(text("inner button B"))
                    .padding(8)
                    .on_press(Message::EmptyButtonEmit),
                text(format!("outer recorded at t = {:.2}s", self.elapsed)).size(12),
                text_input("Placeholder", &self.input_value)
                    .on_input(Message::InputValueUpdate)
                    .size(12),
                row![inner_cached0_1, inner_cached1_1, inner_cached0_3].spacing(15)
            ]
            .spacing(8),
        )
        .padding(20)
        .style(|_t| container::Style {
            background: Some(Color::from_rgb(0.92, 0.94, 0.99).into()),
            text_color: Some(Color::BLACK),
            border: Border {
                color: Color::from_rgb(0.2, 0.3, 0.6),
                width: 1.5,
                radius: 10.0.into(),
            },
            ..Default::default()
        });

        let outer_cached = Cached::new(self.outer_cache.clone(), card)
            .transform(outer_transform)
            .supersample(self.supersample)
            .pixel_snap(self.snap_mode)
            .auto_supersample_on_motion(self.auto_motion_ss)
            .compositing_mode(self.compositing);

        let ss_label = if self.supersample > 1.0 {
            format!("Supersample: {:.0}×", self.supersample)
        } else {
            "Supersample: 1×".to_string()
        };
        let snap_label = match self.snap_mode {
            PixelSnap::Auto => "Snap: Auto (crisp at rest, smooth moving)",
            PixelSnap::Always => "Snap: Always (crisp, steps)",
            PixelSnap::Never => "Snap: Never (smooth, bicubic)",
            PixelSnap::Hybrid => "Snap: Hybrid (bounds snapped, transform smooth)",
        };
        let motion_ss_label = if self.auto_motion_ss {
            "Motion SS: on (1.5× while moving)"
        } else {
            "Motion SS: off"
        };
        let compositing_label = match self.compositing {
            CompositingMode::Layer => "Compositing: Layer (nested-correct)",
            CompositingMode::Bake => "Compositing: Bake (source order)",
        };

        let pause_label = if self.paused { "Resume" } else { "Pause" };

        let controls = row![
            button(text("Reset animation")).on_press(Message::Reset),
            button(text(pause_label)).on_press(Message::TogglePause),
            button(text("Invalidate outer")).on_press(Message::InvalidateOuter),
            button(text("Invalidate inner")).on_press(Message::InvalidateInner),
            button(text(ss_label)).on_press(Message::ToggleSupersample),
            button(text(snap_label)).on_press(Message::ToggleSnap),
            button(text(motion_ss_label)).on_press(Message::ToggleMotionSS),
            button(text(compositing_label)).on_press(Message::ToggleCompositing),
        ]
        .spacing(10);

        let live_timer = text(format!("Live timer: {:.2}s", self.elapsed)).size(14);

        column![
            text("Cached widget demo — nested compositor layers").size(28),
            text(
                "The outer card slides horizontally; the inner badge bobs vertically. \
                In Layer mode (default) both animate simultaneously without ghosting: the inner's \
                pose is NOT baked into the outer's texture — the compositor applies both \
                transforms in a separate pass. Switch to Bake mode to see the pre-v8 \
                source-order behavior (the inner's pose is baked, so the outer card freezes \
                whenever its cache is fresh)."
            )
            .size(13),
            controls,
            live_timer,
            container(outer_cached)
                .width(Length::Fill)
                .height(300)
                .padding(40),
        ]
        .spacing(15)
        .padding(20)
        .into()
    }
}
