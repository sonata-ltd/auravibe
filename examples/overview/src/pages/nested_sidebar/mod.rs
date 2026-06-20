use iced::{
    Element, Padding,
    widget::{container, row, text},
};
use iced_auravibe::{
    Kit, KitProvider,
    mapper::UIMapper,
    router::{Route, RouteMessage, action::Action, page::PageView},
};

use crate::{
    AppDataRegistry,
    pages::nested_sidebar::{button::ButtonsPage, inputs::InputsPage},
};

pub mod button;
pub mod inputs;

pub struct NestedSidebar<KitP: KitProvider> {
    router: Route<KitP>,
    kit: Box<dyn for<'a> Kit<'a, Message>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(usize),
    RouteUpdate(RouteMessage),
}

impl<KitP: KitProvider + Send + Sync> PageView for NestedSidebar<KitP> {
    type Message = Message;
    type NavOptions = ();

    const OVERRIDE_PARENT_STYLING: bool = true;

    fn new(
        kit: Box<dyn for<'k> iced_auravibe::Kit<'k, Self::Message>>,
        registry: iced_auravibe::registry::Registry,
    ) -> Self {
        let binding = &registry.shared::<AppDataRegistry<KitP>>().unwrap();
        let provider = &binding.read().provider;

        let mut router = Route::<KitP>::new(registry, provider.clone());
        router.add::<ButtonsPage>("Buttons");
        router.add::<InputsPage>("Inputs");

        Self { router, kit }
    }

    fn update(
        &mut self,
        msg: Self::Message,
    ) -> iced_auravibe::router::action::Action<Self::Message> {
        match msg {
            Message::Navigate(id) => self.router.navigate_id(id),
            Message::RouteUpdate(msg) => {
                return Action::task(self.router.update(msg).map(Message::RouteUpdate));
            }
        }

        Action::none()
    }

    fn view<'a>(&'a self) -> iced::Element<'a, Self::Message> {
        let kit = UIMapper::new(&self.kit);
        let content = self.router.content().0.map(Message::RouteUpdate);
        let current = self.router.current();

        let mut sidebar_items: Vec<Element<'_, Message>> = vec![
            container(text("Nested sidebar"))
                .padding(Padding::from([15, 5]))
                .into(),
        ];

        sidebar_items.extend(self.router.labels().map(|(id, label)| {
            kit.button()
                .label(label)
                .width(iced::Length::Fill)
                .hier(if current == Some(id) {
                    iced_auravibe::definition::button::props::ButtonHierarchy::Secondary
                } else {
                    iced_auravibe::definition::button::props::ButtonHierarchy::Tertiary
                })
                .on_press(Message::Navigate(id))
                .into()
        }));

        row![
            kit.sidebar(sidebar_items).width(200).show_header(true),
            container(content).padding(15)
        ]
        .into()
    }
}
