use iced::{
    Color, Element, Length, Subscription, Task, theme,
    widget::{container, row},
};

use iced_auravibe::{
    Kit, KitProvider,
    appstate::AppState,
    definition::button::props::ButtonHierarchy,
    kit::sonata::SonataProvider,
    mapper::UIMapper,
    router::{Route, RouteMessage},
};

use crate::pages::{
    buttons::ButtonsPage, content_stack::ContentStackPage, example_page::ExamplePage,
    inputs::InputsPage,
};

mod pages;

fn main() -> iced::Result {
    iced::application(
        move || AppData::new(SonataProvider),
        AppData::update,
        AppData::view,
    )
    .style(|_, _| theme::Style {
        background_color: Color::WHITE,
        text_color: Color::BLACK,
    })
    .subscription(AppData::subscription)
    .run()
}

struct AppData<KitP: KitProvider> {
    route: Route<KitP>,
    uikit: Box<dyn for<'k> Kit<'k, Message>>,
}

#[derive(Clone)]
pub enum Message {
    Navigate(usize),
    RouteUpdate(RouteMessage),
}

impl<KitP: KitProvider> AppData<KitP> {
    fn new(provider: KitP) -> (Self, Task<Message>) {
        let uikit = provider.provide::<Message>();

        let app_state = AppState::new();
        let mut route = Route::<KitP>::new(app_state, provider.clone());

        route.add::<ButtonsPage>("Buttons");
        route.add::<InputsPage>("Inputs");
        route.add::<ExamplePage>("Example Page");
        route.add::<ContentStackPage>("Content Stack");

        (Self { route, uikit }, Task::none())
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![self.route.subscribe().map(Message::RouteUpdate)])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(id) => self.route.navigate_id(id),
            Message::RouteUpdate(msg) => {
                return self.route.update(msg).map(Message::RouteUpdate);
            }
        }

        Task::none()
    }

    fn kit_mapper(&self) -> UIMapper<'_, Message> {
        UIMapper::new(&self.uikit)
    }

    fn view(&self) -> Element<'_, Message> {
        let kit = self.kit_mapper();
        let content = self.route.content().map(Message::RouteUpdate);
        let current = self.route.current();

        let sidebar_items: Vec<Element<'_, Message>> = self
            .route
            .labels()
            .map(|(id, label)| {
                kit.button()
                    .label(label)
                    .width(Length::Fill)
                    .hier(if current == Some(id) {
                        ButtonHierarchy::Secondary
                    } else {
                        ButtonHierarchy::Tertiary
                    })
                    .on_press(Message::Navigate(id))
                    .into()
            })
            .collect();

        row![
            kit.sidebar(sidebar_items).width(200),
            container(content).padding(15)
        ]
        .into()
    }
}
