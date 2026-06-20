use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::appstate::AppState;

#[derive(Clone)]
pub struct Registry(Arc<RwLock<Inner>>);

type RegisterableData = Box<dyn Any + Send + Sync>;

struct Inner {
    shared: HashMap<TypeId, RegisterableData>,
    snapshots: HashMap<TypeId, RegisterableData>,
}

impl Registry {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Inner {
            shared: HashMap::new(),
            snapshots: HashMap::new(),
        })))
    }

    pub fn register<T: Send + Sync + 'static>(&self, value: T) -> AppState<T> {
        let handle = AppState::new(value);
        self.0
            .write()
            .unwrap()
            .shared
            .insert(TypeId::of::<T>(), Box::new(handle.clone()));
        handle
    }

    pub fn shared<T: Send + Sync + 'static>(&self) -> Option<AppState<T>> {
        if let Some(app_state) = self.0.read().unwrap().shared.get(&TypeId::of::<T>()) {
            app_state.downcast_ref::<AppState<T>>().cloned()
        } else {
            None
        }
    }

    pub fn shared_or_insert_with<T: Send + Sync + 'static>(
        &self,
        f: impl FnOnce() -> T,
    ) -> AppState<T> {
        let mut inner = self.0.write().unwrap();
        inner
            .shared
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(AppState::new(f())))
            .downcast_ref::<AppState<T>>()
            .expect("The key TypeId::<T> must contain AppState<T>")
            .clone()
    }

    pub(crate) fn save_snapshot(&self, page: TypeId, snap: RegisterableData) {
        self.0.write().unwrap().snapshots.insert(page, snap);
    }

    pub(crate) fn take_snapshot(&self, page: TypeId) -> Option<RegisterableData> {
        self.0.write().unwrap().snapshots.remove(&page)
    }

    pub(crate) fn drop_snapshot(&self, page: TypeId) {
        self.0.write().unwrap().snapshots.remove(&page);
    }
}
