use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::entry::{Entry, HasName};

#[derive(Debug, Clone)]
pub struct NamedRegistry<T>(Arc<RwLock<HashMap<String, Entry<T>>>>);

impl<T> NamedRegistry<T>
where
    T: HasName,
{
    fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn insert(&self, entry: T) -> bool {
        self.lock()
            .insert(entry.name(), Entry::new(entry))
            .is_some()
    }

    pub fn update(&self, entry: &mut T) where T: Clone {
        if let Some(existing) = self.get(&entry.name()) {
            existing.mutate(|inner| *inner = entry.clone());
        }
    }

    pub fn get(&self, name: &str) -> Option<Entry<T>> {
        self.rlock().get(name).cloned()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.rlock().contains_key(name)
    }

    pub fn mutate<F>(&self, key: &str, f: F) -> bool
    where
        F: FnOnce(&mut T),
    {
        if let Some(entry) = self.get(key) {
            entry.mutate(f);
            return true;
        }
        false
    }

    fn rlock(&self) -> RwLockReadGuard<'_, HashMap<String, Entry<T>>> {
        self.0.read().unwrap()
    }

    pub fn lock(&self) -> RwLockWriteGuard<'_, HashMap<String, Entry<T>>> {
        self.0.write().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[derive(Debug, Clone, PartialEq)]
    struct InnerMock {
        name: String,
        value: i32,
    }

    impl HasName for InnerMock {
        fn name(&self) -> String {
            self.name.clone()
        }
    }

    #[rstest]
    fn test_insert_and_get() {
        let reg = NamedRegistry::new();
        let item = InnerMock {
            name: "alpha".into(),
            value: 10,
        };

        reg.insert(item.clone());
        let fetched = reg.get("alpha").unwrap().lock().clone();

        assert_eq!(fetched, item);
        assert!(reg.contains("alpha"));
    }

    #[rstest]
    fn test_mutate_existing_entry() {
        let reg = NamedRegistry::new();
        reg.insert(InnerMock {
            name: "beta".into(),
            value: 1,
        });

        let ok = reg.mutate("beta", |v| v.value += 41);

        assert!(ok);
        let val = reg.get("beta").unwrap().lock().value;
        assert_eq!(val, 42);
    }

    #[rstest]
    fn test_mutate_nonexistent_entry_returns_false() {
        let reg = NamedRegistry::<InnerMock>::new();
        let result = reg.mutate("nope", |_| {});
        assert!(!result);
    }

    #[rstest]
    fn test_update_replaces_existing_value() {
        let reg = NamedRegistry::new();

        reg.insert(InnerMock {
            name: "gamma".into(),
            value: 5,
        });
        reg.update(&mut InnerMock {
            name: "gamma".into(),
            value: 100,
        });

        let val = reg.get("gamma").unwrap().lock().value;
        assert_eq!(val, 100);
    }

    #[rstest]
    fn test_contains_and_get() {
        let reg = NamedRegistry::new();
        reg.insert(InnerMock {
            name: "delta".into(),
            value: 7,
        });

        assert!(reg.contains("delta"));
        assert!(!reg.contains("unknown"));

        let val = reg.get("delta").unwrap().lock().value;
        assert_eq!(val, 7);
    }

    #[rstest]
    fn test_multiple_entries_concurrent_mutation() {
        use std::thread;

        let reg = Arc::new(NamedRegistry::new());
        reg.insert(InnerMock {
            name: "x".into(),
            value: 0,
        });
        reg.insert(InnerMock {
            name: "y".into(),
            value: 100,
        });

        let reg1 = reg.clone();
        let t1 = thread::spawn(move || {
            reg1.mutate("x", |v| v.value += 10);
        });

        let reg2 = reg.clone();
        let t2 = thread::spawn(move || {
            reg2.mutate("y", |v| v.value -= 50);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        assert_eq!(reg.get("x").unwrap().lock().value, 10);
        assert_eq!(reg.get("y").unwrap().lock().value, 50);
    }

    #[rstest]
    fn test_lock_and_rlock_consistency() {
        let reg = NamedRegistry::new();
        reg.insert(InnerMock {
            name: "omega".into(),
            value: 9,
        });

        {
            let map = reg.rlock();
            assert!(map.contains_key("omega"));
        }

        {
            let mut map = reg.lock();
            map.insert(
                "phi".into(),
                Entry::new(InnerMock {
                    name: "phi".into(),
                    value: 33,
                }),
            );
        }

        assert!(reg.contains("phi"));
    }
}
