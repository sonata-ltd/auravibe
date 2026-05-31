use iced::{
    Element,
    widget::{Space, column},
};
use iced_auravibe::{
    Kit,
    appstate::AppState,
    mapper::UIMapper,
    router::{action::Action, page::PageView},
};

#[derive(Clone)]
pub enum Message {
    InputChanged(String),
    UnblockInput,
}

pub struct InputsPage {
    disable_input: bool,
    input_content: String,

    app_state: AppState,
    kit: Box<dyn for<'k> Kit<'k, Message>>,
}

pub struct NavOptions {
    pub disable_input: bool,
}

impl PageView for InputsPage {
    type Message = Message;
    type NavOptions = NavOptions;

    fn new(kit: Box<dyn for<'k> Kit<'k, Self::Message>>, app_state: AppState) -> Self {
        let input_content = app_state.read().some_state.clone();

        Self {
            disable_input: false,
            input_content,
            app_state,
            kit,
        }
    }

    fn update(&mut self, msg: Message) -> Action<Message> {
        match msg {
            Message::InputChanged(i) => {
                self.input_content = i.clone();
                self.app_state.write().some_state = i;
            }
            Message::UnblockInput => self.disable_input = false,
        }

        Action::none()
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        let kit = UIMapper::new(&self.kit);

        let mut input = kit.input("Placeholder", &self.input_content);
        if !self.disable_input {
            input = input.on_input(Message::InputChanged);
        }

        column![input, {
            let el: Element<Message> = if self.disable_input {
                kit.button()
                    .label("Unblock")
                    .on_press(Message::UnblockInput)
                    .into()
            } else {
                Space::new().into()
            };

            el
        }]
        .into()
    }

    fn on_nav(&mut self, options: Self::NavOptions) {
        self.disable_input = options.disable_input;
    }
}
