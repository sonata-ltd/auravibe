//! Demonstrates the `Cached` widget from `iced_auravibe`.
//!
//! The colored card on screen is rasterized into a `TextureCache`
//! **once** and then composited every frame under a sinusoidal
//! translate transformation. The label inside the card displays the
//! elapsed time at the moment the cache was last (re)recorded; since
//! the cache is fresh, the timestamp does **not** update each frame —
//! demonstrating that the cached texture is frozen until invalidated.
//!
//! Buttons:
//!  - `Reset` — resets the elapsed timer (the *transform* parameter)
//!    without invalidating the cache, so the card stops moving but
//!    its rasterized contents stay the same.
//!  - `Invalidate cache` — forces a re-record: the next frame, the
//!    cache is repopulated with the current `elapsed` label.
use std::time::Instant;

use iced::{
    Border, Color, Element, Length, Subscription, Task, TextureCache, Theme, Transformation,
    widget::{button, column, container, row, text},
    window,
};
use iced_auravibe::cached::Cached;

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
    cache: TextureCache,
    supersample: f32,
    snap: bool,
}

#[derive(Clone, Debug)]
enum Message {
    Tick(Instant),
    Reset,
    InvalidateCache,
    ToggleSupersample,
    ToggleSnap,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                started_at: None,
                elapsed: 0.0,
                cache: TextureCache::new(),
                // Record the card at 2× resolution so its text stays sharp
                // while it slides across fractional device-pixel positions.
                supersample: 2.0,
                snap: false,
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
                let started_at = *self.started_at.get_or_insert(now);
                self.elapsed = now.duration_since(started_at).as_secs_f32();
            }
            Message::Reset => {
                self.started_at = None;
                self.elapsed = 0.0;
            }
            Message::InvalidateCache => {
                self.cache.invalidate();
            }
            Message::ToggleSupersample => {
                self.supersample = if self.supersample > 1.0 { 1.0 } else { 2.0 };
            }
            Message::ToggleSnap => {
                self.snap = !self.snap;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Sinusoidal horizontal translation, ±100 logical pixels.
        let amplitude = 100.0;
        let speed = 1.5;
        let dx = (self.elapsed * speed).sin() * amplitude;
        let transform = Transformation::translate(dx, 0.0);

        // The "expensive" content: a colored card with a few rows of text
        // and buttons. In a real app this could be a complex chart, a
        // multi-border component, a paragraph of styled text, etc.
        let card = container(
            column![
                text("Cached card").size(22),
                text("(rasterized once, composited cheaply)").size(12),
                button(text("inner button A")).padding(8),
                button(text("inner button B")).padding(8),
                text(format!("recorded at t = {:.2}s", self.elapsed)).size(12),
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

        let cached = Cached::new(self.cache.clone(), card)
            .transform(transform)
            .supersample(self.supersample)
            .snap_to_pixels(self.snap);

        let ss_label = if self.supersample > 1.0 {
            format!("Supersample: {:.0}× (sharp)", self.supersample)
        } else {
            "Supersample: 1× (blurry in motion)".to_string()
        };
        let snap_label = if self.snap {
            "Pixel snap: on (crisp, steppy)"
        } else {
            "Pixel snap: off"
        };

        let controls = row![
            button(text("Reset animation")).on_press(Message::Reset),
            button(text("Invalidate cache")).on_press(Message::InvalidateCache),
            button(text(ss_label)).on_press(Message::ToggleSupersample),
            button(text(snap_label)).on_press(Message::ToggleSnap),
        ]
        .spacing(10);

        let live_timer = text(format!("Live timer: {:.2}s", self.elapsed)).size(14);

        column![
            text("Cached widget demo").size(28),
            text(
                "The card is rasterized into a texture once. Each frame, \
                only a transform is applied to the cached texture — the \
                underlying widget tree is not re-rendered. The timestamp \
                inside the card freezes at the moment of the last record."
            )
            .size(13),
            controls,
            live_timer,
            container(cached)
                .width(Length::Fill)
                .height(300)
                .padding(40),
        ]
        .spacing(15)
        .padding(20)
        .into()
    }
}
