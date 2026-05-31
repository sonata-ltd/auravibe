use iced::{Element, Task, widget::text};

#[derive(Clone, Debug)]
pub enum Message {
    Increase,
    Decrease,
}

#[derive(Default)]
pub struct Counter {
    value: i32,
}

impl Counter {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increase => self.value += 1,
            Message::Decrease => self.value -= 1,
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        text(&self.value).into()
    }
}
