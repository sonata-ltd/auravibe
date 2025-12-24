use std::marker::PhantomData;

use crate::{Kit, definition::button::main::UiButton};

// pub mod button;

// pub type MessageMapper<AppMsg> = Arc<dyn Fn(KitMessage) -> AppMsg + Send + Sync + 'static>;

pub struct UIMapper<'a, Message> {
    kit: &'a Box<dyn Kit<Message>>,
    _marker: PhantomData<Message>,
}

impl<'a, Message> UIMapper<'a, Message>
where
    Message: Clone + 'static,
{
    pub fn new(kit: &'a Box<dyn Kit<Message>>) -> Self {
        Self {
            kit,
            _marker: PhantomData,
        }
    }

    pub fn button(&self, label: impl Into<String>, on_press: Message) -> UiButton<'a, Message> {
        UiButton::new(&self.kit, label, on_press)
    }

    // pub fn build(&self, btn: UiButton<Message>) -> Element<'static, Message> {
    //     self.kit.constr_button(btn)
    // }

    // pub fn raw(&self, el: Element<'static, KitMessage>) -> Element<'a, AppMsg> {
    //     let mapper = self.get_mapper();
    //     el.map(move |kmsg| (mapper)(kmsg))
    // }
}
