use iced::{
    Element,
    widget::{Space, column},
};
use iced_auravibe::{
    Kit,
    appstate::AppState,
    mapper::UIMapper,
    registry::Registry,
    router::{action::Action, page::PageView},
};

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    UnblockInput,
}

pub struct InputsPage {
    disable_input: bool,
    input_content: String,
    shared: AppState<String>,

    kit: Box<dyn for<'k> Kit<'k, Message>>,
}

pub struct NavOptions {
    pub disable_input: bool,
}

impl PageView for InputsPage {
    type Message = Message;
    type NavOptions = NavOptions;

    fn new(kit: Box<dyn for<'k> Kit<'k, Self::Message>>, registry: Registry) -> Self {
        let shared = registry.shared_or_insert_with(|| String::new());
        let input_content = shared.read().clone();

        Self {
            disable_input: false,
            input_content,
            shared,
            kit,
        }
    }

    fn update(&mut self, msg: Message) -> Action<Message> {
        match msg {
            Message::InputChanged(i) => {
                *self.shared.write() = i.clone();
                self.input_content = i;
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

    fn on_resume(&mut self) {
        println!("on resume");
        self.input_content = self.shared.read().clone();
    }
}
