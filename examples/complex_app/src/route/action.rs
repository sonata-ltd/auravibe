use std::any::{Any, TypeId};

use iced::Task;

use crate::pages::PageView;

pub type NavOptions = Option<Box<dyn Any + Send>>;

#[derive(Debug)]
pub struct Nav {
    pub(crate) target: TypeId,
    pub(crate) options: NavOptions,
}

pub struct Action<M> {
    pub(crate) task: Task<M>,
    pub(crate) nav: Option<Nav>,
}

impl<M> Action<M> {
    pub fn none() -> Self {
        Self {
            task: Task::none(),
            nav: None,
        }
    }

    // Only task
    pub fn task(task: Task<M>) -> Self {
        Self { task, nav: None }
    }

    // Navigation without options
    pub fn nav<P: PageView>() -> Self
    where
        P::NavOptions: Send + 'static,
    {
        Self {
            task: Task::none(),
            nav: Some(Nav {
                target: TypeId::of::<P>(),
                options: None,
            }),
        }
    }

    // Navigation with options
    pub fn nav_with<P: PageView>(options: P::NavOptions) -> Self
    where
        P::NavOptions: Send + 'static,
    {
        Self {
            task: Task::none(),
            nav: Some(Nav {
                target: TypeId::of::<P>(),
                options: Some(Box::new(options)),
            }),
        }
    }

    // Task + navigation without options
    pub fn task_and_nav<P: PageView>(task: Task<M>) -> Self
    where
        P::NavOptions: Send + 'static,
    {
        Self {
            task,
            nav: Some(Nav {
                target: TypeId::of::<P>(),
                options: None,
            }),
        }
    }

    // Task + navigation with options
    pub fn task_and_nav_with<P: PageView>(mut self, options: P::NavOptions) -> Self
    where
        P::NavOptions: Send + 'static,
    {
        self.nav = Some(Nav {
            target: TypeId::of::<P>(),
            options: Some(Box::new(options)),
        });

        self
    }
}
