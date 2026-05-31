use iced::Subscription;

use crate::{Kit, appstate::AppState, router::action::Action};

pub trait PageView: 'static {
    type Message: Clone + Send + 'static;
    type NavOptions: Send + 'static;

    fn new(kit: Box<dyn for<'k> Kit<'k, Self::Message>>, app_state: AppState) -> Self;

    fn sub(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    // Global subscription for any background tasks
    // that will be performed throughout the entire application
    fn bg_sub(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(&mut self, msg: Self::Message) -> Action<Self::Message>;
    fn view<'a>(&'a self) -> iced::Element<'a, Self::Message>;
    fn on_nav(&mut self, _options: Self::NavOptions) {}
}
