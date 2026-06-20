use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use iced::{
    Element, Event, Subscription, Task, event,
    mouse::{self, Button},
    widget::Space,
};

use crate::{
    KitProvider,
    registry::Registry,
    router::{
        action::Nav,
        history::{NavigationChange, NavigationHistory},
        page::{LifeCycle, PageView},
    },
};

pub mod action;
pub mod history;
pub mod page;

pub trait AnyMessage: std::fmt::Debug + Any + Send {
    fn clone_box(&self) -> Box<dyn AnyMessage>;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Clone + Send + 'static + std::fmt::Debug> AnyMessage for T {
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

#[derive(Debug, Clone)]
pub enum RouteMessage {
    Page {
        page_id: usize,
        inner: Box<dyn AnyMessage>,
    },
    Navigate(NavigationChange),
}

impl RouteMessage {
    fn wrap<M: Clone + Send + 'static + std::fmt::Debug>(id: usize, msg: M) -> Self {
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
    fn on_nav_dyn(&mut self, options: Option<Box<dyn Any + Send>>);

    fn snapshot_dyn(&self) -> Option<Box<dyn Any + Send + Sync>>;
    fn restore_dyn(&mut self, snapshot: Box<dyn Any + Send + Sync>);
    fn on_suspend_dyn(&mut self);
    fn on_resume_dyn(&mut self);
    fn is_override_parent_container(&self) -> bool;
}

struct PageSlot<P: PageView> {
    page: P,
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

    fn snapshot_dyn(&self) -> Option<Box<dyn Any + Send + Sync>> {
        self.page.snapshot()
    }

    fn restore_dyn(&mut self, snapshot: Box<dyn Any + Send + Sync>) {
        self.page.restore(snapshot);
    }

    fn on_resume_dyn(&mut self) {
        self.page.on_resume();
    }

    fn on_suspend_dyn(&mut self) {
        self.page.on_suspend();
    }

    fn is_override_parent_container(&self) -> bool {
        self.page.is_override_parent_container()
    }
}

pub struct PageEntry<KitP> {
    factory: Box<dyn Fn(&KitP, &Registry) -> Box<dyn DynPage>>,
    instance: Option<Box<dyn DynPage>>,
    lifecycle: LifeCycle,
    override_parent_styling: bool,
    suspended: bool,
    type_id: TypeId,
    label: String,
}

// Route
#[allow(dead_code)]
pub struct Route<P> {
    pub pages: Vec<PageEntry<P>>,
    pub index: HashMap<TypeId, usize>,
    pub current: Option<usize>,
    pub history: NavigationHistory,

    pub provider: P,
    pub registry: Registry,
}

#[allow(dead_code)]
impl<KitP: KitProvider> Route<KitP> {
    pub fn new(registry: Registry, provider: KitP) -> Self {
        Self {
            pages: vec![],
            index: HashMap::new(),
            current: None,
            history: NavigationHistory::with_capacity(50),
            provider,
            registry,
        }
    }

    pub fn registry(&self) -> Registry {
        self.registry.clone()
    }

    pub fn add<P: PageView>(&mut self, label: &str) -> &mut Self {
        let id = self.pages.len();

        let factory: Box<dyn Fn(&KitP, &Registry) -> Box<dyn DynPage>> =
            Box::new(|provider: &KitP, registry: &Registry| {
                let kit = provider.provide::<P::Message>();
                Box::new(PageSlot {
                    page: P::new(kit, registry.clone()),
                })
            });

        let instance = if P::EAGER {
            Some(factory(&self.provider, &self.registry))
        } else {
            None
        };

        self.pages.push(PageEntry {
            factory,
            instance,
            lifecycle: P::LIFECYCLE,
            override_parent_styling: P::OVERRIDE_PARENT_STYLING,
            suspended: false,
            type_id: TypeId::of::<P>(),
            label: label.to_string(),
        });

        self.index.insert(TypeId::of::<P>(), id);
        self
    }

    pub fn navigate_id(&mut self, id: usize) {
        if id < self.pages.len() {
            self.set_current(id);
            let removed = self.history.push_history(id);
            self.cleanup_snapshots(&removed);
        }
    }

    pub fn navigate_page<P: PageView>(&mut self) {
        if let Some(&id) = self.index.get(&TypeId::of::<P>()) {
            self.set_current(id);
            let removed = self.history.push_history(id);
            self.cleanup_snapshots(&removed);
        }
    }

    fn navigate_by_type_id(&mut self, nav: Nav) {
        if let Some(&id) = self.index.get(&nav.target) {
            self.set_current(id);

            if let Some(instance) = self.pages[id].instance.as_mut() {
                instance.on_nav_dyn(nav.options);
            }

            let removed = self.history.push_history(id);
            self.cleanup_snapshots(&removed);
        }
    }

    pub fn current(&self) -> Option<usize> {
        self.current
    }

    pub fn set_current(&mut self, to: usize) {
        if self.current == Some(to) {
            return;
        }

        if let Some(from) = self.current {
            match self.pages[from].lifecycle {
                LifeCycle::Suspend => {
                    if let Some(instance) = self.pages[from].instance.as_mut() {
                        instance.on_suspend_dyn();
                    }

                    self.pages[from].suspended = true;
                }
                LifeCycle::Drop => {
                    if let Some(instance) = self.pages[from].instance.take() {
                        if let Some(snapshot) = instance.snapshot_dyn() {
                            let type_id = self.pages[from].type_id;
                            self.registry.save_snapshot(type_id, snapshot);
                        }
                    }

                    self.pages[from].suspended = false;
                }
            }
        }

        if self.pages[to].instance.is_none() {
            let mut instance = (self.pages[to].factory)(&self.provider, &self.registry);
            let type_id = self.pages[to].type_id;

            if let Some(snapshot) = self.registry.take_snapshot(type_id) {
                instance.restore_dyn(snapshot);
            }

            self.pages[to].instance = Some(instance);
        } else if self.pages[to].suspended {
            if let Some(instance) = self.pages[to].instance.as_mut() {
                instance.on_resume_dyn();
            }
        }

        self.pages[to].suspended = false;
        self.current = Some(to);
    }

    fn cleanup_snapshots(&mut self, removed: &[usize]) {
        for &id in removed {
            if !self.history.contains(id) && self.current != Some(id) {
                let type_id = self.pages[id].type_id;
                self.registry.drop_snapshot(type_id);
            }
        }
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

        let bg_sub = self.pages.iter().enumerate().filter_map(|(id, entry)| {
            entry
                .instance
                .as_ref()
                .map(|instance| instance.background_task_dyn(id))
        });

        let current_sub = self.current.and_then(|id| {
            self.pages[id]
                .instance
                .as_ref()
                .map(|instance| instance.subscribe_dyn(id))
        });

        Subscription::batch(bg_sub.chain(current_sub).chain(std::iter::once(event)))
    }

    pub fn update(&mut self, msg: RouteMessage) -> Task<RouteMessage> {
        match msg {
            RouteMessage::Page { page_id, inner } => {
                if let Some(entry) = self.pages.get_mut(page_id) {
                    if let Some(instance) = entry.instance.as_mut() {
                        let (task, nav) = instance.update_dyn(inner, page_id);

                        if let Some(nav) = nav {
                            self.navigate_by_type_id(nav);
                        }

                        return task;
                    }
                }
            }
            RouteMessage::Navigate(NavigationChange::Back) => {
                if let Some(id) = self.history.nav_back() {
                    self.set_current(id);
                }
            }
            RouteMessage::Navigate(NavigationChange::Forward) => {
                if let Some(id) = self.history.nav_forward() {
                    self.set_current(id);
                }
            }
        }

        Task::none()
    }

    pub fn content<'b>(&'b self) -> (Element<'b, RouteMessage>, bool) {
        if let Some(id) = self.current {
            if let Some(instance) = self.pages[id].instance.as_ref() {
                return (
                    instance.view_dyn(id),
                    instance.is_override_parent_container(),
                );
            }
        }

        (Space::new().into(), false)

        // match self
        //     .current
        //     .and_then(|id| self.pages[id].instance.as_ref().map(|instance| instance))
        // {
        //     Some(instance) => (
        //         instance.view_dyn(id),
        //         instance.is_override_parent_container(),
        //     ),
        //     None => Space::new().into(),
        // }
    }

    pub fn labels(&self) -> impl Iterator<Item = (usize, &str)> {
        self.pages
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.label.as_str()))
    }
}
