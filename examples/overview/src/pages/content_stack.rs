use std::sync::Arc;

use iced::Length::{Fill, Shrink};
use iced::widget::{container, row, scrollable, space, text};
use iced::{Alignment, Color, Padding};
use iced::{Element, widget::column};
use iced_auravibe::appstate::AppState;
use iced_auravibe::kit::sonata::utils::text::text::TextStyle;
use iced_auravibe::kit::sonata::utils::widgets::multi_border::MultiBorderExt;
use iced_auravibe::router::action::Action;
use iced_auravibe::router::page::PageView;
use iced_auravibe::{Kit, definition::button::props::ButtonHierarchy, mapper::UIMapper};

#[derive(Clone)]
pub enum Message {
    Back,
    Next,
    ShowBorders,
    None,
    PathToBinaryUpdate(String),
    ArchUpdate(String),
}

pub struct ContentStackPage {
    kit: Box<dyn for<'a> Kit<'a, Message>>,
    content_stack_index: usize,
    back_button_active: bool,
    next_button_active: bool,
    show_borders: bool,

    path_to_binary: String,
    runtime_version: String,
    vendor: String,
    java_home: String,
    architecture: String,
}

impl PageView for ContentStackPage {
    type Message = Message;
    type NavOptions = ();

    fn new(kit: Box<dyn for<'k> Kit<'k, Self::Message>>, _: AppState) -> Self {
        Self {
            kit,
            content_stack_index: 0,
            back_button_active: false,
            next_button_active: true,
            show_borders: false,
            path_to_binary: String::from("/home/quartix/downloads/java1/bin"),
            runtime_version: String::new(),
            vendor: String::new(),
            java_home: String::new(),
            architecture: String::from("x86"),
        }
    }

    fn update(&mut self, msg: Message) -> Action<Message> {
        match msg {
            Message::Back => {
                if self.content_stack_index >= 1 {
                    self.content_stack_index -= 1;
                }
            }
            Message::Next => {
                if self.content_stack_index < 2 {
                    self.content_stack_index += 1;
                }
            }
            Message::ShowBorders => {
                self.show_borders = !self.show_borders;
                self.content_stack_index = 0;
            }
            Message::None => {}
            Message::PathToBinaryUpdate(s) => self.path_to_binary = s,
            Message::ArchUpdate(s) => self.architecture = s,
        }

        self.back_button_active = self.content_stack_index > 0;
        self.next_button_active = self.content_stack_index < 1;

        Action::none()
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        let kit = Arc::new(UIMapper::new(&self.kit));

        let content_stack = kit.window("Add External Runtime").with_children(
            self.content_stack_index,
            [
                screen1(&self, kit.clone()).into(),
                screen2(&self, kit.clone()).into(),
            ]
            .into(),
        );

        column![
            row![
                kit.button()
                    .label("Back")
                    .on_press_maybe(self.back_button_active.then_some(Message::Back))
                    .width(iced::Length::Fill),
                kit.button()
                    .label("Next")
                    .on_press_maybe(self.next_button_active.then_some(Message::Next))
                    .width(iced::Length::Fill),
                kit.button()
                    .label("Show widget borders")
                    .on_press(Message::ShowBorders)
                    .width(Shrink)
                    .hier(ButtonHierarchy::Secondary)
            ]
            .spacing(10),
            container({
                let el: Element<'_, Message>;

                if self.show_borders {
                    el = content_stack
                        .outer_border(1.0, Color::from_rgb8(240, 12, 120))
                        .into();
                } else {
                    el = content_stack.into();
                }

                el
            })
            .align_y(Alignment::Center)
            .height(Fill)
        ]
        .spacing(15)
        .into()
    }
}

fn screen1<'a>(
    data: &'a ContentStackPage,
    kit: Arc<UIMapper<'a, Message>>,
) -> Element<'a, Message> {
    column![
        column![
            kit.input("/home/...", &data.path_to_binary)
                .label("Path To Binary")
                .hint("Path to runtime executable file")
                .on_input(Message::PathToBinaryUpdate),
        ]
        .padding(15),
        row![
            space().width(Fill),
            kit.button()
                .label("Cancel")
                .hier(ButtonHierarchy::Secondary)
                .on_press(Message::None),
            kit.button().label("Apply").on_press(Message::Next),
        ]
        .padding(15)
        .spacing(10),
    ]
    .into()
}

fn screen2<'a>(
    data: &'a ContentStackPage,
    kit: Arc<UIMapper<'a, Message>>,
) -> Element<'a, Message> {
    let header_text = TextStyle::TextMdSm;
    let header_text_size = header_text.get_size();

    let paragraph_text = TextStyle::TextSmR;
    let paragraph_text_size = paragraph_text.get_size();

    scrollable(column![
        column![
            column![
                text("Detected Properties")
                    .font(header_text.build_font())
                    .size(header_text_size)
                    .width(Fill)
                    .center(),
                text("Please verify the automatically detected Java runtime properties.")
                    .font(paragraph_text.build_font())
                    .size(paragraph_text_size)
                    .center(),
            ]
            .padding(Padding::top(Padding::default(), 10))
            .spacing(10),
            column![
                kit.input("25.0.1", &data.runtime_version)
                    .label("Runtime Version"),
                kit.input("GraalVM EE", &data.vendor).label("Vendor"),
                kit.input("/usr/lib/jvm/java-25-openjdk", &data.java_home)
                    .label("Java Home"),
                kit.input("x86", &data.architecture)
                    .label("Architecture")
                    .on_input(Message::ArchUpdate),
            ]
            .spacing(15),
        ]
        .spacing(20)
        .padding(15),
        row![
            space().width(Fill),
            kit.button()
                .label("Cancel")
                .hier(ButtonHierarchy::Secondary)
                .on_press(Message::Back),
            kit.button().label("Apply").on_press(Message::None),
        ]
        .padding(15)
        .spacing(10)
    ])
    .into()
}
