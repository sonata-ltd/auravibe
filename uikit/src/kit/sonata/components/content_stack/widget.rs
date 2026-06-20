use iced::{
    Element, Event, Length, Point, Rectangle, Size, TextureCache, Transformation, Vector,
    advanced::{
        Layout, Shell, Widget,
        graphics::core::TextureRecordMode,
        layer::LayerSlot,
        layout::{Limits, Node},
        mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
    widget::{self, Id},
    window,
};

use std::{f32, sync::Arc, time::Instant};

use crate::kit::sonata::utils::animations::{
    composition::composite_geometry,
    spring::{Spring, SpringParams},
};

const MARGIN: u32 = 0;

pub struct ContentStack<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    id: Option<Id>,
    childs: Vec<Element<'a, Message, Theme, Renderer>>,
    fixed_width: Option<f32>,
    max_height: f32,
    active_index_req: usize,
}

#[derive(Debug)]
struct Animation {
    spring: Spring,
    is_animating: bool,
    last_tick: Option<Instant>,
}

#[derive(Debug)]
struct State {
    content_index: ContentIndex,
    animation: Animation,
    page_slots: Vec<Arc<LayerSlot>>,
    frame_builded_layouts: Vec<usize>,
}

#[derive(Debug)]
struct ContentIndex {
    next: Option<usize>,
    current: usize,
}

impl State {
    fn new(childs_len: usize) -> Self {
        let mut page_slots = Vec::new();
        page_slots.resize_with(childs_len, || LayerSlot::new(TextureCache::new()));

        Self {
            content_index: ContentIndex {
                next: None,
                current: 0,
            },
            animation: Animation {
                spring: Spring::new(SpringParams::default(), 0.0),
                is_animating: false,
                last_tick: None,
            },
            page_slots,
            frame_builded_layouts: Vec::new(),
        }
    }
}

impl<'a, Message, Theme, Renderer> ContentStack<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(active_index_req: usize) -> Self {
        Self::from_vec(Vec::new(), active_index_req)
    }
    pub fn with_capacity(capacity: usize, current_index: usize) -> Self {
        Self::from_vec(Vec::with_capacity(capacity), current_index)
    }

    pub fn with_children(
        current_index: usize,
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let iterator = children.into_iter();

        Self::with_capacity(iterator.size_hint().0, current_index).extend(iterator)
    }

    pub fn from_vec(
        children: Vec<Element<'a, Message, Theme, Renderer>>,
        active_index: usize,
    ) -> Self {
        Self {
            id: None,
            childs: children,
            fixed_width: None,
            max_height: f32::INFINITY,
            active_index_req: active_index,
        }
    }

    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let child = child.into();
        let child_size = child.as_widget().size_hint();

        if !child_size.is_void() {
            self.childs.push(child);
        }

        self
    }

    pub fn extend(
        self,
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        children.into_iter().fold(self, Self::push)
    }

    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.fixed_width = Some(width.into());
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ContentStack<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.childs.len()))
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.childs.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(&self.childs);

        let state = tree.state.downcast_mut::<State>();

        state
            .page_slots
            .resize_with(self.childs.len(), || LayerSlot::new(TextureCache::new()));

        let safe_active_index = self
            .active_index_req
            .min(self.childs.len().saturating_sub(1));

        let current_target = state
            .content_index
            .next
            .unwrap_or(state.content_index.current);
        let safe_current_index = current_target.min(self.childs.len().saturating_sub(1));

        if safe_current_index != safe_active_index {
            state.content_index.next = Some(safe_active_index);

            state.animation.spring.set_target(safe_active_index as f32);
            state.animation.is_animating = true;
            state.animation.last_tick = None;

            // Inavidate textures at diff
            let from = state.content_index.current;
            let to = safe_active_index;

            for i in from.min(to)..=from.max(to) {
                if let Some(slot) = state.page_slots.get(i) {
                    slot.cache.invalidate();
                }
            }
        }
    }

    fn size(&self) -> iced::Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(&mut self, tree: &mut tree::Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let stack_width = if let Some(width) = self.fixed_width {
            width
        } else {
            limits.max().width
        };

        let available_max_height = limits.max().height.min(self.max_height);
        let child_limits = Limits::new(
            Size::new(stack_width, 0.0),
            Size::new(stack_width, available_max_height),
        );

        let state = tree.state.downcast_mut::<State>();

        let max_idx = self.childs.len().saturating_sub(1) as f32;
        let spring_pos = state.animation.spring.position.clamp(0.0, max_idx);

        let idx_a = spring_pos.floor() as usize;
        let idx_b = spring_pos.ceil() as usize;

        let mut indicies_to_layout = Vec::new();

        // Measure only that camera seeing
        if idx_a < self.childs.len() {
            indicies_to_layout.push(idx_a);
        }

        if idx_b < self.childs.len() && idx_b != idx_a {
            indicies_to_layout.push(idx_b);
        }

        let mut measured_nodes = Vec::with_capacity(indicies_to_layout.len());
        let mut h_a = 0.0;
        let mut h_b = 0.0;

        for &i in &indicies_to_layout {
            let node = self.childs[i].as_widget_mut().layout(
                &mut tree.children[i],
                renderer,
                &child_limits,
            );

            let h = node.size().height;

            if i == idx_a {
                h_a = h
            };
            if i == idx_b {
                h_b = h
            };

            measured_nodes.push((i, h, node));
        }

        let progress = spring_pos.fract();
        let layout_height = interpolate_height(h_a, h_b, progress).min(self.max_height);

        let mut children_nodes = Vec::with_capacity(self.childs.len());

        for (i, h, node) in measured_nodes {
            let offset_x = i as f32 * stack_width;
            let offset_y = (layout_height - h) / 2.0;

            children_nodes.push(node.move_to(Point::new(offset_x, offset_y)));
        }

        state.frame_builded_layouts = indicies_to_layout;

        Node::with_children(
            Size {
                width: stack_width,
                height: layout_height,
            },
            children_nodes,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_ref::<State>();

        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            let mut layout_children = layout.children();

            for &i in &state.frame_builded_layouts {
                if let Some(child_layout) = layout_children.next() {
                    self.childs[i].as_widget_mut().operate(
                        &mut tree.children[i],
                        child_layout,
                        renderer,
                        operation,
                    );
                }
            }
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
        let state = tree.state.downcast_mut::<State>();

        // if state.animation.is_animating && state.animation.last_tick.is_none() {
        //     shell.request_redraw();
        // }

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            tick_animation(
                &mut state.animation,
                now,
                shell,
                Some(|| {
                    if let Some(next_idx) = state.content_index.next {
                        state.content_index.current = next_idx;
                        state.content_index.next = None;
                    }
                }),
            );
        }

        let bounds = layout.bounds();
        let offset_x = state.animation.spring.position * bounds.width;
        let translated_cursor = translate_cursor(cursor, offset_x);

        let is_redraw = matches!(event, Event::Window(window::Event::RedrawRequested(_)));
        let animating = state.animation.is_animating;

        // Snapshot the visible-page list so we can iterate without
        // holding a borrow on `state`. `frame_builded_layouts` is
        // typically 1–2 entries during a slide.
        let frame_layouts: Vec<usize> = state.frame_builded_layouts.clone();

        let mut layout_children = layout.children();

        for &i in &frame_layouts {
            if let Some(child_layout) = layout_children.next() {
                // Only register page slots while animating — when
                // settled, the page is drawn inline by `draw` and any
                // stale page-slot cache should not appear in the
                // compose pass.
                let pushed_page = if is_redraw {
                    let slide = Transformation::translate(-offset_x, 0.0);
                    let scale = renderer.scale_factor().unwrap_or(1.0);
                    let page_bounds = child_layout.bounds();

                    let geom = composite_geometry(MARGIN, page_bounds, scale, slide);

                    state.page_slots[i].write(|d| {
                        d.transform = geom.transform;
                        d.bounds = geom.cache_bounds;
                        d.clip_bounds = Some(bounds);
                        d.parent_id = None;
                    });

                    if animating {
                        shell.register_layer(state.page_slots[i].clone());
                        shell.push_layer(&state.page_slots[i]);

                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                let snapshot = (!is_redraw).then(|| {
                    (
                        shell.redraw_request(),
                        shell.event_status(),
                        shell.is_layout_invalid(),
                        shell.are_widgets_invalid(),
                    )
                });

                self.childs[i].as_widget_mut().update(
                    &mut tree.children[i],
                    event,
                    child_layout,
                    translated_cursor,
                    renderer,
                    shell,
                    viewport,
                );

                if let Some((redraw_request, event_status, layout_invalid, widgets_invalid)) =
                    snapshot
                {
                    let reacted = shell.redraw_request() != redraw_request
                        || shell.event_status() != event_status
                        || shell.is_layout_invalid() != layout_invalid
                        || shell.are_widgets_invalid() != widgets_invalid;

                    if reacted {
                        state.page_slots[i].cache.invalidate();
                    }
                }

                if pushed_page {
                    shell.pop_layer();
                }
            }
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
        let bounds = layout.bounds();
        let offset_x = state.animation.spring.position * bounds.width;
        let translated_cursor = translate_cursor(cursor, offset_x);

        let mut max_interaction = mouse::Interaction::default();
        let mut layout_children = layout.children();

        for &i in &state.frame_builded_layouts {
            if let Some(child_layout) = layout_children.next() {
                let interaction = self.childs[i].as_widget().mouse_interaction(
                    &tree.children[i],
                    child_layout,
                    translated_cursor,
                    viewport,
                    renderer,
                );
                max_interaction = interaction.max(interaction);
            }
        }

        max_interaction
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        if self.childs.is_empty() {
            return;
        }

        let offset_x = state.animation.spring.position * bounds.width;
        let translated_cursor = translate_cursor(cursor, offset_x);

        let current = state.content_index.current;
        let target = state.content_index.next.unwrap_or(current);
        let min_i = current.min(target);
        let max_i = current.max(target);

        let mut layout_children = layout.children();
        let is_animating = state.animation.is_animating;
        let scale = renderer.scale_factor().unwrap_or(1.0);

        if is_animating {
            for &i in &state.frame_builded_layouts {
                if let Some(child_layout) = layout_children.next() {
                    if i < min_i || i > max_i {
                        continue;
                    }

                    let child_bounds = child_layout.bounds();

                    let slide = Transformation::translate(-offset_x, 0.0);

                    let geom = composite_geometry(MARGIN, child_bounds, scale, slide);

                    renderer.draw_to_texture(
                        TextureRecordMode::Flush,
                        &state.page_slots[i].cache,
                        geom.padded,
                        geom.tex_scale,
                        |r| {
                            r.with_translation(
                                Vector::new(
                                    -child_bounds.x + MARGIN as f32,
                                    -child_bounds.y + MARGIN as f32,
                                ),
                                |r| {
                                    let fake_viewport = Rectangle {
                                        x: child_bounds.x,
                                        y: child_bounds.y,
                                        width: child_bounds.width,
                                        height: child_bounds.height,
                                    };

                                    self.childs[i].as_widget().draw(
                                        &tree.children[i],
                                        r,
                                        theme,
                                        style,
                                        child_layout,
                                        translated_cursor,
                                        &fake_viewport,
                                    );
                                },
                            );
                        },
                    );
                }
            }
        } else {
            let max_idx = self.childs.len().saturating_sub(1) as f32;
            let spring_pos = state.animation.spring.position.clamp(0.0, max_idx);
            let idx_a = spring_pos.floor() as usize;
            let idx_b = spring_pos.ceil() as usize;

            renderer.with_layer(bounds, |renderer| {
                let mut layout_children = layout.children();
                for &i in &state.frame_builded_layouts {
                    if let Some(child_layout) = layout_children.next() {
                        if i != idx_a && i != idx_b {
                            continue;
                        }

                        let cb = child_layout.bounds();
                        let snap_dx = (cb.x * scale).round() / scale - cb.x;
                        let snap_dy = (cb.y * scale).round() / scale - cb.y;

                        renderer.with_translation(
                            Vector::new(snap_dx - offset_x, snap_dy),
                            |renderer| {
                                let child_viewport = Rectangle {
                                    x: viewport.x + offset_x,
                                    y: viewport.y,
                                    width: viewport.width,
                                    height: viewport.height,
                                };
                                self.childs[i].as_widget().draw(
                                    &tree.children[i],
                                    renderer,
                                    theme,
                                    style,
                                    child_layout,
                                    translated_cursor,
                                    &child_viewport,
                                );
                            },
                        );
                    }
                }
            });
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();
        let offset_x = state.animation.spring.position * layout.bounds().width;
        let local_translation = Vector::new(-offset_x, 0.0);

        let mut overlays = Vec::new();
        let mut layout_children = layout.children();

        for (i, (child, child_tree)) in self.childs.iter_mut().zip(&mut tree.children).enumerate() {
            if state.frame_builded_layouts.contains(&i) {
                if let Some(child_layout) = layout_children.next() {
                    if let Some(overlay) = child.as_widget_mut().overlay(
                        child_tree,
                        child_layout,
                        renderer,
                        viewport,
                        translation + local_translation,
                    ) {
                        overlays.push(overlay);
                    }
                }
            }
        }

        if overlays.is_empty() {
            None
        } else {
            Some(overlay::Group::with_children(overlays).overlay())
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ContentStack<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(widget: ContentStack<'a, Message, Theme, Renderer>) -> Self {
        Self::new(widget)
    }
}

#[inline]
fn translate_cursor(cursor: mouse::Cursor, offset_x: f32) -> mouse::Cursor {
    match cursor {
        mouse::Cursor::Available(p) => mouse::Cursor::Available(Point::new(p.x + offset_x, p.y)),
        mouse::Cursor::Unavailable => mouse::Cursor::Unavailable,
        mouse::Cursor::Levitating(p) => mouse::Cursor::Available(Point::new(p.x + offset_x, p.y)),
    }
}

fn tick_animation<Message, F>(
    anim: &mut Animation,
    now: &Instant,
    shell: &mut Shell<'_, Message>,
    after_fn: Option<F>,
) where
    F: FnOnce(),
{
    if !anim.is_animating {
        return;
    }

    let target_dt = 1.0 / 60.0;

    let dt = anim
        .last_tick
        .map(|prev| now.duration_since(prev).as_secs_f32())
        .unwrap_or(0.0)
        .min(target_dt);

    anim.last_tick = Some(*now);
    anim.spring.tick(dt);

    if anim.spring.is_settled() {
        anim.spring.snap();
        anim.is_animating = false;
        anim.last_tick = None;

        if let Some(f) = after_fn {
            f();
        }
    }

    shell.invalidate_layout();
    shell.request_redraw();
}

#[inline]
fn interpolate_height(current: f32, next: f32, pos: f32) -> f32 {
    current + (next - current) * pos
}
