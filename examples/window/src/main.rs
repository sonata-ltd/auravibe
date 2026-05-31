use iced::{
    Alignment, Color, Element, Length, theme,
    widget::{Space, button, column, container, row, scrollable, text, text_input},
};
use iced_auravibe::{Kit, kit::sonata::Sonata, mapper::UIMapper};

// --- СОСТОЯНИЕ ТЕСТОВОГО ПРИЛОЖЕНИЯ ---
pub struct StressTestApp {
    current_index: usize,
    input_value: String,
    uikit: Box<dyn for<'k> Kit<'k, Message>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NextPage,
    PrevPage,
    InputChanged(String),
}

fn main() -> iced::Result {
    iced::application(
        StressTestApp::new,
        StressTestApp::update,
        StressTestApp::view,
    )
    .style(|_, _| theme::Style {
        background_color: Color::WHITE,
        text_color: Color::BLACK,
    })
    .run()
}

impl StressTestApp {
    pub fn new() -> Self {
        Self {
            current_index: 0,
            input_value: String::new(),
            uikit: Box::new(Sonata::new()),
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::NextPage => {
                self.current_index = (self.current_index + 1).min(2); // У нас 3 страницы (0, 1, 2)
            }
            Message::PrevPage => {
                self.current_index = self.current_index.saturating_sub(1);
            }
            Message::InputChanged(val) => {
                self.input_value = val;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let kit = UIMapper::new(&self.uikit);
        // ==========================================
        // СТРАНИЦА 1: Форма с кучей инпутов (Тест фокуса и ввода)
        // ==========================================
        let mut form_column = column![
            text("Регистрация (Стресс-тест)").size(30),
            text("Заполните все поля ниже. Это тестирует layout инпутов.").size(16),
            Space::new().height(20),
        ]
        .spacing(15);

        // Генерируем 10 текстовых полей
        for i in 1..=10 {
            form_column = form_column.push(
                text_input(&format!("Поле ввода #{}", i), &self.input_value)
                    .on_input(Message::InputChanged)
                    .padding(10),
            );
        }

        form_column = form_column.push(
            button(text("Следующая страница ->").size(20).center())
                .width(Length::Fill)
                .padding(15)
                .on_press(Message::NextPage),
        );

        let page1 = container(scrollable(form_column).height(Length::Fill))
            .padding(20)
            .height(Length::Fill);

        // ==========================================
        // СТРАНИЦА 2: Сложный список (Тест вложенности UI)
        // ==========================================
        let mut list_column = column![
            button(text("<- Назад").size(16))
                .padding(10)
                .on_press(Message::PrevPage),
            Space::new().height(10),
            text("Список элементов (50 строк)").size(30),
        ]
        .spacing(10);

        // Генерируем 50 строк (имитация ленты новостей/пользователей)
        for i in 1..=50 {
            list_column = list_column.push(
                row![
                    // Имитация аватарки
                    container(text(format!("{}", i)).center())
                        .width(50.0)
                        .height(50.0),
                    // Имитация текста и описания
                    column![
                        text(format!("Пользователь #{}", i)).size(18),
                        text("Это длинное описание, которое должно переноситься на новые строки и нагружать кэш шрифтов Iced.")
                            .size(14),
                    ]
                    .spacing(5)
                ]
                .spacing(15)
                .align_y(Alignment::Center),
            );
        }

        list_column = list_column.push(
            button(text("К последней странице ->").size(20).center())
                .width(Length::Fill)
                .padding(15)
                .on_press(Message::NextPage),
        );

        let page2 = container(scrollable(list_column).height(Length::Fill))
            .padding(20)
            .height(Length::Fill);

        let massive_text = "Стресс-тест шрифтов! ".repeat(500);

        let page3 = container(
            scrollable(
                column![
                    text("Лицензионное соглашение").size(30),
                    text("Этот текст нагружает cosmic-text максимально сильно."),
                    button(text("<- В начало").size(20).center())
                        .width(Length::Fill)
                        .padding(15)
                        .on_press(Message::PrevPage),
                    text(massive_text).size(15),
                ]
                .spacing(20),
            )
            .height(Length::Fill),
        )
        .padding(20)
        .height(Length::Fill);

        container(kit.window("label").with_children(
            self.current_index,
            [page1.into(), page2.into(), page3.into()].into(),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
    }
}
