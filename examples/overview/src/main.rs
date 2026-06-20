#![windows_subsystem = "windows"]

use iced::{
    Color, Element, Font, Length, Subscription, Task, theme,
    widget::{container, row},
};

use iced_auravibe::{
    Kit, KitProvider,
    definition::button::props::ButtonHierarchy,
    kit::sonata::{SonataProvider, utils::text::FONT_INTER},
    mapper::UIMapper,
    registry::Registry,
    router::{Route, RouteMessage},
};

use crate::pages::{
    buttons::ButtonsPage, content_stack::ContentStackPage, example_page::ExamplePage,
    inputs::InputsPage, nested_sidebar::NestedSidebar,
};

mod pages;

fn main() -> iced::Result {
    iced::application(
        move || AppData::new(SonataProvider, Registry::new()),
        AppData::update,
        AppData::view,
    )
    .style(|_, _| theme::Style {
        background_color: Color::WHITE,
        text_color: Color::BLACK,
    })
    .font(FONT_INTER)
    .default_font(Font::with_family("Inter"))
    .subscription(AppData::subscription)
    .run()
}

struct AppData<KitP: KitProvider> {
    router: Route<KitP>,
    uikit: Box<dyn for<'k> Kit<'k, Message>>,
}

struct AppDataRegistry<KitP: KitProvider> {
    provider: KitP,
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(usize),
    RouteUpdate(RouteMessage),
}

impl<KitP: KitProvider + Send + Sync> AppData<KitP> {
    fn new(provider: KitP, registry: Registry) -> (Self, Task<Message>) {
        let uikit = provider.provide::<Message>();

        registry.register(AppDataRegistry {
            provider: provider.clone(),
        });

        let mut route = Route::<KitP>::new(registry, provider.clone());

        route.add::<ButtonsPage>("Buttons");
        route.add::<InputsPage>("Inputs");
        route.add::<ExamplePage>("Example Page");
        route.add::<ContentStackPage>("Content Stack");
        route.add::<NestedSidebar<KitP>>("Nested Sidebar");

        (
            Self {
                router: route,
                uikit,
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![self.router.subscribe().map(Message::RouteUpdate)])
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

    fn kit_mapper(&self) -> UIMapper<'_, Message> {
        UIMapper::new(&self.uikit)
    }

    fn view(&self) -> Element<'_, Message> {
        let kit = self.kit_mapper();

        let content_options = self.router.content();

        let content = content_options.0.map(Message::RouteUpdate);
        let override_container = content_options.1;
        let current = self.router.current();

        let sidebar_items: Vec<Element<'_, Message>> = self
            .router
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

        row![kit.sidebar(sidebar_items).width(200), {
            let widget;

            if override_container {
                widget = content;
            } else {
                widget = container(content).padding(15).into();
            }

            widget
        }]
        .into()
    }
}
