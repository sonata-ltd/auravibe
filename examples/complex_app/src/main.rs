use iced::{
    Color, Element,
    Length::{self},
    Task, theme,
    widget::container,
};
use iced_auravibe::{KitProvider, kit::sonata::SonataProvider};

use crate::{
    pages::auth::AuthPage,
    route::{Route, RouteMessage},
    state::AppState,
};

mod pages;
mod route;
mod state;

fn main() -> iced::Result {
    iced::application(move || Data::new(SonataProvider), Data::update, Data::view)
        .style(|_, _| theme::Style {
            background_color: Color::WHITE,
            text_color: Color::BLACK,
        })
        .run()
}

struct Data<KitP: KitProvider> {
    router: Route<KitP>,
}

#[derive(Clone)]
pub enum Message {
    Navigate(usize),
    RouteUpdate(RouteMessage),
}

impl<KitP: KitProvider> Data<KitP> {
    fn new(provider: KitP) -> (Self, Task<Message>) {
        let app_state = AppState::new();
        let mut router = Route::<KitP>::new(app_state, provider.clone());

        router.add::<AuthPage>("Auth");
        router.navigate_page::<AuthPage>();

        (Self { router }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(id) => self.router.navigate_id(id),
            Message::RouteUpdate(msg) => {
                return self.router.update(msg).map(Message::RouteUpdate);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content = self.router.content().map(Message::RouteUpdate);

        container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}
