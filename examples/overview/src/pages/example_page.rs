use iced::widget::text;
use iced::{Element, widget::column};
use iced_auravibe::Kit;
use iced_auravibe::appstate::AppState;
use iced_auravibe::router::action::Action;
use iced_auravibe::router::page::PageView;

#[derive(Clone)]
pub enum Message {}

pub struct ExamplePage {}

impl PageView for ExamplePage {
    type Message = Message;
    type NavOptions = ();

    fn new(_: Box<dyn for<'k> Kit<'k, Self::Message>>, _: AppState) -> Self {
        Self {}
    }

    fn update(&mut self, _: Message) -> Action<Message> {
        Action::none()
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        column![text("This is an example page")].spacing(15).into()
    }
}
