use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    /// Toggled to `true` by Flutter via FFI whenever the user interacts.
    pub user_is_active: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user_is_active: Arc::new(AtomicBool::new(false)),
        }
    }
}