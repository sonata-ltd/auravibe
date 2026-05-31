use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone)]
pub struct AppState(Arc<RwLock<AppStateInner>>);

pub struct AppStateInner {
    pub some_state: String,
}

impl AppState {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(AppStateInner {
            some_state: String::from("asdasd"),
        })))
    }

    pub fn read(&self) -> RwLockReadGuard<'_, AppStateInner> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, AppStateInner> {
        self.0.write().unwrap()
    }
}
