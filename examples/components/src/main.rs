use iced::{
    Alignment, Color, Element, Length, Task, theme,
    widget::{column, container},
};
use iced_auravibe::{Kit, kit::sonata::Sonata, mapper::UIMapper};

use crate::number::Counter;

mod number;

fn main() -> iced::Result {
    iced::application(
        move || AppData::new(Sonata::new()),
        AppData::update,
        AppData::view,
    )
    .style(|_, _| theme::Style {
        background_color: Color::WHITE,
        text_color: Color::BLACK,
    })
    .run()
}

struct AppData {
    counter: Counter,
    uikit: Box<dyn for<'a> Kit<'a, Message>>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Counter(number::Message),
}

impl AppData {
    fn new<K>(kit: K) -> (Self, Task<Message>)
    where
        K: for<'a> Kit<'a, Message> + 'static,
    {
        (
            Self {
                counter: Counter::default(),
                uikit: Box::new(kit),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Counter(msg) => {
                let _ = self.counter.update(msg);
            }
        }

        Task::none()
    }

    fn kit_mapper(&self) -> UIMapper<'_, Message> {
        UIMapper::new(&self.uikit)
    }

    fn view(&self) -> Element<'_, Message> {
        let kit = self.kit_mapper();

        container(
            column![
                kit.button()
                    .label("+")
                    .on_press(Message::Counter(number::Message::Increase)),
                self.counter.view().map(|_| unreachable!()),
                kit.button()
                    .label("-")
                    .on_press(Message::Counter(number::Message::Decrease)),
            ]
            .align_x(Alignment::Center)
            .spacing(10),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
    }
}
