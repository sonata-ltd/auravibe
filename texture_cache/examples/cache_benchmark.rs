//! Cache benchmark for `iced_texture_cache`.
//!
//! Left: a heavy static scene moving on a Lissajous path, either wrapped in
//! `Cached` (records once) or drawn directly every frame. Right: the z-order
//! rule, a quad drawn *after* a `Cached` in the same layer lands beneath the
//! texture, while a `stack` gives each child its own layer and draws in order.
//! Bottom: the knobs and the frame-interval statistics.
//!
//! Every knob is in the UI: the cached toggle, the `PixelSnap` mode, the
//! `FilterQuality` tier, the supersample-in-motion switch and the grid size. The only environment
//! variable is `BENCH_LOG=1`, read once in `main`, which prints the statistics
//! line to stderr every 120 frames. Measurements are collected in
//! `BENCHMARKS.md`.

use std::collections::VecDeque;
use std::time::Instant;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, mouse, overlay};
use iced::widget::{Space, checkbox, column, container, pick_list, row, slider, stack, text};
use iced::{
    Background, Border, Color, Event, Length, Padding, Point, Rectangle, Size, Subscription, Theme,
    Vector,
};
use iced_texture_cache::{Element, FilterQuality, PixelSnap, Renderer, TextureCache, cached};

/// Side of one scene cell in logical pixels.
const CELL: f32 = 14.0;
/// Columns at start-up; rows are always two thirds of the columns.
const DEFAULT_COLS: u32 = 30;
/// Size of the z-order panels.
const PANEL: Size = Size::new(260.0, 200.0);
/// Size of the quads and the cached box in the z-order panels.
const BOX: Size = Size::new(120.0, 80.0);

/// `PixelSnap` modes by name, for the pick list.
const SNAP_OPTIONS: [&str; 4] = ["auto", "always", "never", "layout-only"];

/// The `PixelSnap` mode behind a pick-list entry.
fn pixel_snap(name: &str) -> PixelSnap {
    match name {
        "always" => PixelSnap::Always,
        "never" => PixelSnap::Never,
        "layout-only" => PixelSnap::LayoutOnly,
        _ => PixelSnap::Auto,
    }
}

/// Reconstruction tiers by name, for the pick list. "auto" leaves the widget
/// on the renderer's own choice.
const FILTER_OPTIONS: [&str; 4] = ["auto", "catmull-rom", "bilinear", "snap"];

/// The tier behind a pick-list entry; `None` inherits the renderer's.
fn filter_quality(name: &str) -> Option<FilterQuality> {
    match name {
        "catmull-rom" => Some(FilterQuality::CatmullRom),
        "bilinear" => Some(FilterQuality::Bilinear),
        "snap" => Some(FilterQuality::Snap),
        _ => None,
    }
}

fn main() -> iced::Result {
    // `RUST_LOG=info` shows the adapter and the surface-format choice.
    env_logger::init();

    let log = std::env::var("BENCH_LOG").is_ok_and(|value| value == "1");

    iced::application(
        move || Benchmark::new(log),
        Benchmark::update,
        Benchmark::view,
    )
    .subscription(Benchmark::subscription)
    .title("iced_texture_cache: cache benchmark")
    .run()
}

/// Application messages.
#[derive(Debug, Clone, Copy)]
enum Message {
    /// A frame timestamp; the scene moves and the statistics advance.
    Tick(Instant),
    /// The "cached" checkbox.
    CachedToggled(bool),
    /// The `PixelSnap` pick list.
    SnapSelected(&'static str),
    /// The `FilterQuality` pick list.
    FilterSelected(&'static str),
    /// The "supersample in motion" checkbox.
    SupersampleToggled(bool),
    /// The grid slider (columns).
    ColumnsChanged(u32),
}

/// The benchmark state: the knobs, the caches and the frame history.
struct Benchmark {
    cols: u32,
    cached_on: bool,
    snap: &'static str,
    filter: &'static str,
    supersample_in_motion: bool,
    log: bool,
    scene_cache: TextureCache,
    /// One cache per `Cached` widget: two widgets sharing a handle would
    /// re-record each other whenever their sizes differ.
    zorder_cache: TextureCache,
    zorder_stack_cache: TextureCache,
    start: Instant,
    now: Instant,
    frames: VecDeque<Instant>,
    ticks: u64,
}

impl Benchmark {
    fn new(log: bool) -> Self {
        let now = Instant::now();
        Self {
            cols: DEFAULT_COLS,
            cached_on: true,
            snap: SNAP_OPTIONS[0],
            filter: FILTER_OPTIONS[0],
            supersample_in_motion: false,
            log,
            scene_cache: TextureCache::new(),
            zorder_cache: TextureCache::new(),
            zorder_stack_cache: TextureCache::new(),
            start: now,
            now,
            frames: VecDeque::with_capacity(121),
            ticks: 0,
        }
    }

    fn rows(&self) -> u32 {
        self.cols * 2 / 3
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                self.now = now;
                self.frames.push_back(now);
                while self.frames.len() > 120 {
                    let _ = self.frames.pop_front();
                }
                self.ticks += 1;
                if self.log && self.ticks.is_multiple_of(120) {
                    eprintln!("cached={} {}", self.cached_on, self.stats_line());
                }
            }
            Message::CachedToggled(on) => self.cached_on = on,
            Message::SnapSelected(name) => self.snap = name,
            Message::FilterSelected(name) => self.filter = name,
            Message::SupersampleToggled(on) => self.supersample_in_motion = on,
            Message::ColumnsChanged(cols) => self.cols = cols,
        }
    }

    fn subscription(_state: &Self) -> Subscription<Message> {
        iced::window::frames().map(Message::Tick)
    }

    fn offset(&self) -> Vector {
        let t = (self.now - self.start).as_secs_f32();
        Vector::new((t * 1.7).sin() * 40.0, (t * 1.1).cos() * 24.0)
    }

    /// Average and 99th percentile of the last 120 frame intervals, in ms.
    fn frame_stats(&self) -> (f32, f32) {
        let mut intervals: Vec<f32> = self
            .frames
            .iter()
            .zip(self.frames.iter().skip(1))
            .map(|(a, b)| (*b - *a).as_secs_f32() * 1000.0)
            .collect();
        if intervals.is_empty() {
            return (0.0, 0.0);
        }
        let avg = intervals.iter().sum::<f32>() / intervals.len() as f32;
        intervals.sort_by(f32::total_cmp);
        let p99 = intervals[((intervals.len() - 1) as f32 * 0.99) as usize];
        (avg, p99)
    }

    fn stats_line(&self) -> String {
        let (avg, p99) = self.frame_stats();
        format!(
            "{}x{} cells · frame avg {avg:.2} ms · p99 {p99:.2} ms · records: scene {} · zorder {} / {}",
            self.cols,
            self.rows(),
            self.scene_cache.record_count(),
            self.zorder_cache.record_count(),
            self.zorder_stack_cache.record_count(),
        )
    }

    /// The heavy scene on its dark panel, cached or drawn directly.
    fn scene_panel(&self) -> Element<'_, Message> {
        let offset = self.offset();
        let (cols, rows) = (self.cols as usize, self.rows() as usize);

        let scene: Element<'_, Message> = if self.cached_on {
            let scene = cached(self.scene_cache.clone(), heavy_scene(cols, rows))
                .translate(offset)
                .pixel_snap(pixel_snap(self.snap))
                .supersample_in_motion(self.supersample_in_motion);

            match filter_quality(self.filter) {
                Some(quality) => scene.filter_quality(quality).into(),
                None => scene.into(),
            }
        } else {
            container(heavy_scene(cols, rows))
                .padding(Padding {
                    top: 24.0 + offset.y,
                    left: 40.0 + offset.x,
                    right: 0.0,
                    bottom: 0.0,
                })
                .into()
        };

        container(scene)
            .width(Length::Fixed(cols as f32 * CELL + 120.0))
            .height(Length::Fixed(rows as f32 * CELL + 80.0))
            .padding(if self.cached_on {
                Padding {
                    top: 24.0,
                    left: 40.0,
                    right: 0.0,
                    bottom: 0.0,
                }
            } else {
                Padding::ZERO
            })
            .style(panel_style)
            .into()
    }

    /// Red quad, cached green box, blue quad, in one layer and in a `stack`.
    fn zorder_panels(&self) -> Element<'_, Message> {
        let same_layer = Overlap::new(cached(self.zorder_cache.clone(), zorder_content()).into());

        let stacked = stack![
            container(Space::new().width(BOX.width).height(BOX.height))
                .style(|_| solid(Color::from_rgb(0.9, 0.3, 0.3))),
            container(cached(self.zorder_stack_cache.clone(), zorder_content())).padding(Padding {
                top: 30.0,
                left: 40.0,
                right: 0.0,
                bottom: 0.0,
            }),
            container(
                container(Space::new().width(BOX.width).height(BOX.height))
                    .style(|_| solid(Color::from_rgb(0.3, 0.3, 0.9)))
            )
            .padding(Padding {
                top: 60.0,
                left: 80.0,
                right: 0.0,
                bottom: 0.0,
            }),
        ]
        .width(PANEL.width)
        .height(PANEL.height);

        column![
            text("same layer: blue drawn after → beneath texture"),
            same_layer,
            text("stack: blue drawn after → on top"),
            stacked,
        ]
        .spacing(6)
        .into()
    }

    fn controls(&self) -> Element<'_, Message> {
        let knobs = row![
            checkbox(self.cached_on)
                .label("cached")
                .on_toggle(Message::CachedToggled),
            row![
                text("snap"),
                pick_list(SNAP_OPTIONS, Some(self.snap), Message::SnapSelected),
            ]
            .spacing(6),
            row![
                text("filter"),
                pick_list(FILTER_OPTIONS, Some(self.filter), Message::FilterSelected),
            ]
            .spacing(6),
            checkbox(self.supersample_in_motion)
                .label("supersample in motion")
                .on_toggle(Message::SupersampleToggled),
            row![
                text(format!("grid {}×{}", self.cols, self.rows())),
                slider(10..=120_u32, self.cols, Message::ColumnsChanged).width(200),
            ]
            .spacing(6),
        ]
        .spacing(20);

        column![knobs, text(self.stats_line())].spacing(8).into()
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            row![self.scene_panel(), self.zorder_panels()].spacing(20),
            self.controls(),
        ]
        .spacing(16)
        .padding(16)
        .into()
    }
}

/// `cols × rows` coloured, numbered cells: expensive to draw, static in
/// content.
fn heavy_scene(cols: usize, rows: usize) -> Element<'static, Message> {
    let rows = (0..rows).map(|r| {
        let cells = (0..cols).map(|c| {
            let i = r * cols + c;
            let hue = (i % 12) as f32 / 12.0;
            container(text(format!("{}", i % 100)).size(8))
                .width(Length::Fixed(CELL))
                .height(Length::Fixed(CELL))
                .center_x(Length::Fixed(CELL))
                .center_y(Length::Fixed(CELL))
                .style(move |_| container::Style {
                    background: Some(Background::Color(Color::from_rgb(
                        0.3 + 0.5 * hue,
                        0.6 - 0.3 * hue,
                        0.5,
                    ))),
                    border: Border {
                        radius: 3.0.into(),
                        width: 1.0,
                        color: Color::BLACK,
                    },
                    ..container::Style::default()
                })
                .into()
        });
        iced::widget::Row::with_children(cells).into()
    });
    iced::widget::Column::with_children(rows).into()
}

/// The green box the z-order panels cache.
fn zorder_content() -> Element<'static, Message> {
    container(text("cached").size(14))
        .width(Length::Fixed(BOX.width))
        .height(Length::Fixed(BOX.height))
        .center_x(Length::Fixed(BOX.width))
        .center_y(Length::Fixed(BOX.height))
        .style(|_| solid(Color::from_rgb(0.3, 0.8, 0.3)))
        .into()
}

/// A container style with a solid background and nothing else.
fn solid(color: Color) -> container::Style {
    container::Style {
        background: Some(Background::Color(color)),
        ..container::Style::default()
    }
}

/// The dark panel behind the scene.
fn panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.18))),
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: Color::from_rgb(0.4, 0.4, 0.45),
        },
        ..container::Style::default()
    }
}

/// Draws a red quad, then its child at (40, 30), then a blue quad at (80, 60),
/// all in the *same* layer. Demonstrates the documented z-order rule.
///
/// The child is a real tree child: it gets its own `Tree` node, is diffed
/// with `diff_children`, and every `Widget` method that touches the child is
/// forwarded, including `mouse_interaction`, `operate` and `overlay`, which
/// a wrapper must not forget even when its child happens to be inert.
struct Overlap<'a> {
    child: Element<'a, Message>,
}

impl<'a> Overlap<'a> {
    fn new(child: Element<'a, Message>) -> Self {
        Self { child }
    }

    fn child_layout(layout: Layout<'_>) -> Layout<'_> {
        layout
            .children()
            .next()
            .expect("Overlap lays out exactly one child")
    }
}

impl Widget<Message, Theme, Renderer> for Overlap<'_> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.child)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.child));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(PANEL.width), Length::Fixed(PANEL.height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .child
            .as_widget_mut()
            .layout(
                &mut tree.children[0],
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(220.0, 170.0)),
            )
            .move_to(Point::new(40.0, 30.0));
        layout::Node::with_children(PANEL, vec![child])
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.child.as_widget_mut().update(
            &mut tree.children[0],
            event,
            Self::child_layout(layout),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
        use iced::advanced::Renderer as _;

        let origin = layout.bounds().position();
        let quad = |x: f32, y: f32| Quad {
            bounds: Rectangle::new(Point::new(origin.x + x, origin.y + y), BOX),
            ..Quad::default()
        };

        renderer.fill_quad(quad(0.0, 0.0), Color::from_rgb(0.9, 0.3, 0.3));

        self.child.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            Self::child_layout(layout),
            cursor,
            viewport,
        );

        renderer.fill_quad(quad(80.0, 60.0), Color::from_rgb(0.3, 0.3, 0.9));
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.child.as_widget().mouse_interaction(
            &tree.children[0],
            Self::child_layout(layout),
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
        self.child.as_widget_mut().operate(
            &mut tree.children[0],
            Self::child_layout(layout),
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
        self.child.as_widget_mut().overlay(
            &mut tree.children[0],
            Self::child_layout(layout),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a> From<Overlap<'a>> for Element<'a, Message> {
    fn from(widget: Overlap<'a>) -> Self {
        Element::new(widget)
    }
}
