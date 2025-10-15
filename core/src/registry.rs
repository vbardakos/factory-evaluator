use std::{hash::Hash, sync::Arc};

use dashmap::DashSet;

/// SafeSet
pub(crate) struct Registry<T> {
    inner: DashSet<Arc<T>>,
}

impl<T> Registry<T>
where
    T: Hash + Eq + Send + Sync + 'static,
{
    fn new() -> Self {
        Self {
            inner: DashSet::new(),
        }
    }

    pub(crate) fn insert(&self, value: T) -> bool {
        self.inner.insert(Arc::new(value))
    }

    pub(crate) fn contains(&self, value: &T) -> bool {
        self.inner.contains(value)
    }

    pub(crate) fn remove(&self, value: &T) -> bool {
        self.inner.remove(value).is_some()
    }
}
