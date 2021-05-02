use super::LayerHandle;

use std::sync::atomic::{self, AtomicI64};

impl LayerHandle {
    /// Generate a new LayerHandle which is known to be unique in this process.
    pub fn generate() -> Self {
        static COUNTER: AtomicI64 = AtomicI64::new(0);
        let id = COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
        Self { id }
    }
}
