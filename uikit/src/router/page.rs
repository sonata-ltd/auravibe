use std::any::Any;

use iced::Subscription;

use crate::{Kit, registry::Registry, router::action::Action};

pub enum LifeCycle {
    Drop,
    Suspend,
}

pub trait PageView: 'static {
    type Message: Clone + Send + 'static + std::fmt::Debug;
    type NavOptions: Send + 'static;

    const LIFECYCLE: LifeCycle = LifeCycle::Drop;
    const EAGER: bool = false;
    const OVERRIDE_PARENT_STYLING: bool = false;

    // Init new page.
    fn new(kit: Box<dyn for<'k> Kit<'k, Self::Message>>, registry: Registry) -> Self;

    // Create subscription that will be passed to
    // main `Iced` subscription in `main`.
    fn sub(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    // Global subscription for any background tasks
    // that will be performed throughout the entire application.
    fn bg_sub(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    // Update logic inside this page.
    fn update(&mut self, msg: Self::Message) -> Action<Self::Message>;

    // Widget composition.
    fn view<'a>(&'a self) -> iced::Element<'a, Self::Message>;

    // Navigation options to this page.
    fn on_nav(&mut self, _options: Self::NavOptions) {}

    // Function that will be executed on page suspension.
    fn on_suspend(&mut self) {}

    // Function that will be executed on page resume.
    fn on_resume(&mut self) {}

    // That will be saved on page leave.
    fn snapshot(&self) -> Option<Box<dyn Any + Send + Sync>> {
        None
    }

    // That will be restored on page load.
    fn restore(&mut self, _snapshot: Box<dyn Any + Send + Sync>) {}

    fn is_override_parent_container(&self) -> bool {
        Self::OVERRIDE_PARENT_STYLING
    }
}
