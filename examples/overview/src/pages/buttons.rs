use crate::pages::inputs::InputsPage;
use crate::pages::inputs::NavOptions as InputsNavOptions;
use iced::{Element, widget::column};
use iced_auravibe::appstate::AppState;
use iced_auravibe::router::action::Action;
use iced_auravibe::router::page::PageView;
use iced_auravibe::{Kit, definition::button::props::ButtonHierarchy, mapper::UIMapper};

#[derive(Clone)]
pub enum Message {
    ActionPressed,
    NavigateInputs,
}

pub struct ButtonsPage {
    kit: Box<dyn for<'a> Kit<'a, Message>>,
}

impl PageView for ButtonsPage {
    type Message = Message;
    type NavOptions = ();

    fn new(kit: Box<dyn for<'k> Kit<'k, Self::Message>>, _: AppState) -> Self {
        Self { kit }
    }

    fn update(&mut self, msg: Message) -> Action<Message> {
        match msg {
            Message::ActionPressed => {
                println!("Button Pressed")
            }
            Message::NavigateInputs => {
                return Action::nav_with::<InputsPage>(InputsNavOptions {
                    disable_input: true,
                });
            }
        }

        Action::none()
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        let kit = UIMapper::new(&self.kit);

        column![
            kit.button()
                .label("Go To Inputs")
                .on_press(Message::NavigateInputs),
            kit.button()
                .label("Action")
                .hier(ButtonHierarchy::Secondary)
                .on_press(Message::ActionPressed),
            kit.button()
                .label("Action")
                .hier(ButtonHierarchy::Tertiary)
                .on_press(Message::ActionPressed),
        ]
        .spacing(15)
        .into()
    }
}
