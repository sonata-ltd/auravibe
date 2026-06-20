use std::time::Instant;

use iced::advanced::Shell;

pub mod composition;
pub mod spring;

#[derive(Debug)]
pub struct Animation {
    pub spring: spring::Spring,
    pub is_animating: bool,
    pub last_tick: Option<Instant>,
}

pub fn tick_animation<Message, F>(
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
