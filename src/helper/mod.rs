use std::sync::Arc;

use iced::Element;

use crate::{Kit, KitMessage, kit::sonata::Sonata};

// pub mod button;

pub type MessageMapper<AppMsg> = Arc<dyn Fn(KitMessage) -> AppMsg + Send + Sync + 'static>;

pub struct UIMapper<'a, AppMsg> {
    kit: Box<&'a Sonata>,
    mapper: MessageMapper<AppMsg>,
}

impl<'a, AppMsg: 'static> UIMapper<'a, AppMsg> {
    pub fn new(
        kit: &'a Sonata,
        mapper: impl Fn(KitMessage) -> AppMsg + Send + Sync + 'static,
    ) -> Self {
        Self {
            kit: Box::from(kit),
            mapper: Arc::new(mapper),
        }
    }

    pub fn get_mapper(&self) -> MessageMapper<AppMsg> {
        Arc::clone(&self.mapper)
    }

    pub fn raw(&self, el: Element<'static, KitMessage>) -> Element<'a, AppMsg> {
        let mapper = self.get_mapper();
        el.map(move |kmsg| (mapper)(kmsg))
    }
}
