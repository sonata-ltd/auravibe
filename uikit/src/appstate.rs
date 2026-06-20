use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct AppState<T>(Arc<RwLock<T>>);

impl<T> AppState<T> {
    pub fn new(state: T) -> Self {
        Self(Arc::new(RwLock::new(state)))
    }

    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().unwrap()
    }
}

impl<T> Clone for AppState<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
