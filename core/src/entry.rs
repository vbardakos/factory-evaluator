use std::sync::{Arc, Mutex, MutexGuard, Weak};

#[derive(Debug)]
pub struct Entry<T>(Arc<Mutex<T>>);

impl<T> Clone for Entry<T> {
    fn clone(&self) -> Self {
        Entry(self.0.clone())
    }
}

impl<T> Entry<T>
where
    T: HasName,
{
    pub fn new(inner: T) -> Self {
        Self(Arc::new(Mutex::new(inner)))
    }

    pub fn update(&self, inner: &mut T) {
        let mut guard = self.lock();
        std::mem::swap(&mut *guard, inner);
    }

    pub fn mutate<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(&mut self.lock())
    }

    pub fn arc(&self) -> Arc<Mutex<T>> {
        Arc::clone(&self.0)
    }

    pub fn weak(&self) -> Weak<Mutex<T>> {
        Arc::downgrade(&self.0)
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.0.lock().unwrap()
    }
}

pub trait HasName {
    fn name(&self) -> String;
}

impl<T> HasName for Entry<T>
where
    T: HasName,
{
    fn name(&self) -> String {
        self.0.try_lock().unwrap_or(self.lock()).name()
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
    fn test_new_creates_entry() {
        let data = InnerMock {
            name: "alpha".into(),
            value: 10,
        };
        let entry = Entry::new(data.clone());

        let locked = entry.lock();
        assert_eq!(*locked, data);
    }

    #[rstest]
    fn test_mutate() {
        let entry = Entry::new(InnerMock {
            name: "bar".into(),
            value: 5,
        });

        entry.mutate(|v| v.value += 10);

        let locked = entry.lock();
        assert_eq!(locked.value, 15);
    }

    #[rstest]
    fn test_multiple_mutations() {
        let entry = Entry::new(InnerMock {
            name: "baz".into(),
            value: 0,
        });

        entry.mutate(|v| v.value += 1);
        entry.mutate(|v| v.value += 2);
        entry.mutate(|v| v.value += 3);

        assert_eq!(entry.lock().value, 6);
    }

    #[rstest]
    fn test_clone() {
        let entry = Entry::new(InnerMock {
            name: "qux".into(),
            value: 1,
        });
        let entry_clone = entry.clone();

        entry_clone.mutate(|v| v.value += 9);

        let original_value = entry.lock().value;
        let cloned_value = entry_clone.lock().value;

        assert_eq!(original_value, cloned_value);
        assert_eq!(original_value, 10);
    }

    #[rstest]
    fn test_update_replaces_value() {
        let entry = Entry::new(InnerMock {
            name: "gamma".into(),
            value: 42,
        });
        let new_data = InnerMock {
            name: "delta".into(),
            value: 99,
        };

        entry.update(&mut new_data.clone());

        let locked = entry.lock();
        assert_eq!(*locked, new_data);
    }

    #[rstest]
    fn test_arc_returns_same_arc() {
        let entry = Entry::new(InnerMock {
            name: "epsilon".into(),
            value: 7,
        });
        let arc1 = entry.arc();
        let arc2 = entry.arc();

        // arcs should point to same obj
        assert!(Arc::ptr_eq(&arc1, &arc2));

        let locked = arc1.lock().unwrap();
        assert_eq!(locked.value, 7);
    }

    #[rstest]
    fn test_weakref_upgrade_and_drop() {
        let entry = Entry::new(InnerMock {
            name: "zeta".into(),
            value: 123,
        });
        let weak_ref = entry.weak();

        // arcs alive
        assert!(weak_ref.upgrade().is_some());

        drop(entry);

        // arcs dropped
        assert!(weak_ref.upgrade().is_none());
    }

    #[rstest]
    fn test_concurrent_mutation() {
        use std::thread;

        let entry = Arc::new(Entry::new(InnerMock {
            name: "eta".into(),
            value: 0,
        }));

        let mut handles = vec![];
        for _ in 0..5 {
            let e = entry.clone();
            handles.push(thread::spawn(move || {
                e.mutate(|v| v.value += 1);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(entry.lock().value, 5);
    }

    #[rstest]
    fn test_lock_allows_direct_access() {
        let entry = Entry::new(InnerMock {
            name: "theta".into(),
            value: 50,
        });

        {
            let mut guard = entry.lock();
            guard.value *= 2;
        }

        assert_eq!(entry.lock().value, 100);
    }
}
