use alloc::rc::Rc;

/// Cache a function's output and mutation for Rc's with the same pointer value.
///
/// /!\ The caller must not evaluate Rc's that have been created during the cache's lifetime
/// because the memory address might have been freed to take ownership of inner memory.
pub(crate) struct RcCacheMut<T: Clone, U, F: FnMut(&mut T) -> U> {
    cache: Vec<(*const T, Rc<T>, U)>,
    func: F,
}

impl<T: Clone, U, F: FnMut(&mut T) -> U> RcCacheMut<T, U, F> {
    pub(crate) fn new(func: F) -> Self {
        Self { cache: Default::default(), func }
    }

    pub(crate) fn apply(&mut self, val: &mut Rc<T>) -> &U {
        let ptr = Rc::as_ptr(val);

        let cache_slot = {
            if let Some(cache_idx) = self.cache.iter().position(|(idx, _, _)| *idx == ptr) {
                let cached_slot = &mut self.cache[cache_idx];
                *val = cached_slot.1.clone();
                cached_slot
            } else {
                let result = (self.func)(Rc::make_mut(val));
                self.cache.push_mut((ptr, val.clone(), result))
            }
        };

        &cache_slot.2
    }
}

/// Cache a function's output for Rc's with the same pointer value.
///
/// /!\ The caller must not evaluate Rc's that have been created during the cache's lifetime
/// because the memory address might have been freed to take ownership of inner memory.
pub(crate) struct RcCacheOwned<T: Clone, U, F: FnMut(T) -> U> {
    cache: Vec<(*const T, U)>,
    func: F,
}

impl<T: Clone, U, F: FnMut(T) -> U> RcCacheOwned<T, U, F> {
    pub(crate) fn new(func: F) -> Self {
        Self { cache: Default::default(), func }
    }

    pub(crate) fn apply(&mut self, val: Rc<T>) -> &U {
        let ptr = Rc::as_ptr(&val);

        let cache_slot = {
            if let Some(cache_idx) = self.cache.iter().position(|(idx, _)| *idx == ptr) {
                &mut self.cache[cache_idx]
            } else {
                let result = (self.func)(Rc::unwrap_or_clone(val));
                self.cache.push_mut((ptr, result))
            }
        };

        &cache_slot.1
    }
}
