use super::*;
use iced::{
    Alignment, Element,
    Length::{self},
    widget::{container, text},
};
use iced_auravibe::mapper::UIMapper;

#[derive(Clone)]
pub enum Message {
    ScHandleChange(String),
    DiscordUsernameChange(String),
    ReasonChange(String),
    RequestSubmit,
    Back,
}

pub struct AuthPage {
    sc_handle: String,
    discord_username: String,
    reason: String,

    stack_index: usize,

    kit: Box<dyn for<'a> Kit<'a, Message>>,
}

impl PageView for AuthPage {
    type Message = Message;
    type NavOptions = ();

    fn new(kit: Box<dyn for<'k> Kit<'k, Self::Message>>, _: AppState) -> Self {
        Self {
            sc_handle: String::new(),
            discord_username: String::new(),
            reason: String::new(),

            stack_index: 0,

            kit,
        }
    }

    fn update(&mut self, msg: Message) -> Action<Message> {
        match msg {
            Message::ScHandleChange(s) => self.sc_handle = s,
            Message::DiscordUsernameChange(s) => self.discord_username = s,
            Message::ReasonChange(s) => self.reason = s,
            Message::RequestSubmit => self.stack_index += 1,
            Message::Back => self.stack_index -= 1,
        }

        Action::none()
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        let kit = UIMapper::new(&self.kit);

        container(text("todo"))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}
