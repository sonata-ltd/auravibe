use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use iced::{
    Element, Event, Subscription, Task, event,
    mouse::{self, Button},
    widget::Space,
};
use iced_auravibe::KitProvider;

use crate::{
    pages::PageView,
    route::{
        action::Nav,
        history::{NavigationChange, NavigationHistory},
    },
    state::AppState,
};

pub mod action;
pub mod history;

pub trait AnyMessage: Any + Send {
    fn clone_box(&self) -> Box<dyn AnyMessage>;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Clone + Send + 'static> AnyMessage for T {
    fn clone_box(&self) -> Box<dyn AnyMessage> {
        Box::new(self.clone())
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl Clone for Box<dyn AnyMessage> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Unified type for any message

#[derive(Clone)]
pub enum RouteMessage {
    Page {
        page_id: usize,
        inner: Box<dyn AnyMessage>,
    },
    Navigate(NavigationChange),
}

impl RouteMessage {
    fn wrap<M: Clone + Send + 'static>(id: usize, msg: M) -> Self {
        Self::Page {
            page_id: id,
            inner: Box::new(msg),
        }
    }
}

// Dynamic page trait

pub trait DynPage {
    fn update_dyn(
        &mut self,
        msg: Box<dyn AnyMessage>,
        id: usize,
    ) -> (Task<RouteMessage>, Option<Nav>);
    fn subscribe_dyn(&self, id: usize) -> Subscription<RouteMessage>;
    fn background_task_dyn(&self, id: usize) -> Subscription<RouteMessage>;
    fn view_dyn(&self, id: usize) -> Element<'_, RouteMessage>;
    fn label(&self) -> &str;
    fn on_nav_dyn(&mut self, options: Option<Box<dyn Any + Send>>);
}

struct PageSlot<P: PageView> {
    page: P,
    label: String,
}

impl<P: PageView> DynPage for PageSlot<P> {
    fn update_dyn(
        &mut self,
        msg: Box<dyn AnyMessage>,
        id: usize,
    ) -> (Task<RouteMessage>, Option<Nav>) {
        let any_box: Box<dyn Any> = msg.into_any();
        match any_box.downcast::<P::Message>() {
            Ok(concrete) => {
                let action = self.page.update(*concrete);
                let task = action.task.map(move |m| RouteMessage::wrap(id, m));
                (task, action.nav)
            }
            Err(_) => (Task::none(), None),
        }
    }

    fn subscribe_dyn(&self, id: usize) -> Subscription<RouteMessage> {
        self.page
            .sub()
            .with(id)
            .map(|(id, m)| RouteMessage::wrap(id, m))
    }

    fn background_task_dyn(&self, id: usize) -> Subscription<RouteMessage> {
        self.page
            .bg_sub()
            .with(id)
            .map(|(id, m)| RouteMessage::wrap(id, m))
    }

    fn view_dyn(&self, id: usize) -> Element<'_, RouteMessage> {
        self.page.view().map(move |m| RouteMessage::wrap(id, m))
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn on_nav_dyn(&mut self, options: Option<Box<dyn Any + Send>>) {
        match options {
            Some(opts_box) => {
                if let Ok(opts) = opts_box.downcast::<P::NavOptions>() {
                    self.page.on_nav(*opts);
                }
            }
            None => {}
        }
    }
}

// Route

pub struct Route<P> {
    pub pages: Vec<Box<dyn DynPage>>,
    pub index: HashMap<TypeId, usize>,
    pub current: Option<usize>,
    pub history: NavigationHistory,

    pub provider: P,
    pub app_state: AppState,
}

impl<KitP: KitProvider> Route<KitP> {
    pub fn new(app_state: AppState, provider: KitP) -> Self {
        Self {
            pages: vec![],
            index: HashMap::new(),
            current: None,
            history: NavigationHistory::with_capacity(50),
            provider,
            app_state,
        }
    }

    pub fn add<P: PageView>(&mut self, label: &str) -> &mut Self {
        let kit = self.provider.provide::<P::Message>();
        let page = P::new(kit, self.app_state.clone());
        let id = self.pages.len();

        self.pages.push(Box::new(PageSlot {
            page,
            label: label.to_string(),
        }));

        self.history.push_history(id);

        self.index.insert(TypeId::of::<P>(), id);

        self
    }

    pub fn navigate_id(&mut self, id: usize) {
        if id < self.pages.len() {
            self.current = Some(id);
            self.history.push_history(id);
        }
    }

    pub fn navigate_page<P: PageView>(&mut self) {
        if let Some(id) = self.index.get(&TypeId::of::<P>()) {
            self.current = Some(*id);
            self.history.push_history(*id);
        }
    }

    fn navigate_by_type_id(&mut self, nav: Nav) {
        if let Some(id) = self.index.get(&nav.target) {
            self.current = Some(*id);
            self.pages[*id].on_nav_dyn(nav.options);
            self.history.push_history(*id);
        }
    }

    pub fn current(&self) -> Option<usize> {
        self.current
    }

    pub fn subscribe(&self) -> Subscription<RouteMessage> {
        let event = event::listen_with(|event, _, _| match event {
            Event::Mouse(mouse::Event::ButtonPressed(btn)) => match btn {
                Button::Back => Some(RouteMessage::Navigate(NavigationChange::Back)),
                Button::Forward => Some(RouteMessage::Navigate(NavigationChange::Forward)),
                _ => None,
            },
            _ => None,
        });

        let bg_sub = self
            .pages
            .iter()
            .enumerate()
            .map(|(id, page)| page.background_task_dyn(id));

        let current_sub = self.current.map(|(id)| self.pages[id].subscribe_dyn(id));

        Subscription::batch(bg_sub.chain(current_sub).chain(std::iter::once(event)))
    }

    pub fn update(&mut self, msg: RouteMessage) -> Task<RouteMessage> {
        match msg {
            RouteMessage::Page { page_id, inner } => {
                if let Some(page) = self.pages.get_mut(page_id) {
                    let (task, nav) = page.update_dyn(inner, page_id);

                    if let Some(nav) = nav {
                        self.navigate_by_type_id(nav);
                    }

                    return task;
                }
            }
            RouteMessage::Navigate(NavigationChange::Back) => {
                self.history.nav_back().map(|id| self.current = Some(id));
            }
            RouteMessage::Navigate(NavigationChange::Forward) => {
                self.history.nav_forward().map(|id| self.current = Some(id));
            }
        }

        Task::none()
    }

    pub fn content<'b>(&'b self) -> Element<'b, RouteMessage> {
        match self.current {
            Some(id) => self.pages[id].view_dyn(id),
            None => Space::new().into(),
        }
    }

    pub fn labels(&self) -> impl Iterator<Item = (usize, &str)> {
        self.pages.iter().enumerate().map(|(i, p)| (i, p.label()))
    }
}
