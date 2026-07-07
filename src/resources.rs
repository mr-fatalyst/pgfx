// Centralized resource handle management.
// IDs come from a monotonic counter and are never reused, so a stale ID
// resolves to None instead of aliasing a newer resource.

use std::collections::HashMap;

pub struct ResourcePool<T> {
    items: HashMap<u32, T>,
    next_id: u32,
}

impl<T> ResourcePool<T> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn insert(&mut self, item: T) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.insert(id, item);
        id
    }

    pub fn get(&self, id: u32) -> Option<&T> {
        self.items.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut T> {
        self.items.get_mut(&id)
    }

    pub fn remove(&mut self, id: u32) -> Option<T> {
        self.items.remove(&id)
    }
}

impl<T> Default for ResourcePool<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_never_reused() {
        let mut pool = ResourcePool::new();
        let a = pool.insert("a");
        pool.remove(a);
        let b = pool.insert("b");
        assert_ne!(a, b);
        assert!(pool.get(a).is_none());
        assert_eq!(pool.get(b), Some(&"b"));
    }
}
