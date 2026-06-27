use alloc::rc::{Rc, Weak};
use alloc::vec::Vec;

// --
// -- struct: RcCacheMut
// --

/// Cache a function's output and mutation for Rc's with the same pointer value.
pub(crate) struct RcCacheMut<T: Clone, U: Clone, F: FnMut(&mut T) -> U> {
    cache: RcIndex<T, (Rc<T>, U)>,
    func: F,
}

impl<T: Clone, U: Clone, F: FnMut(&mut T) -> U> RcCacheMut<T, U, F> {
    pub(crate) fn new(func: F) -> Self {
        Self { cache: Default::default(), func }
    }

    pub(crate) fn apply(&mut self, val: &mut Rc<T>) -> U {
        let key = Rc::downgrade(val);

        if Rc::strong_count(val) == 1 {
            (self.cache.pop(&key))
                .map(|(cached_val, result)| {
                    *val = cached_val;
                    result
                })
                .unwrap_or_else(|| (self.func)(Rc::make_mut(val)))
        } else {
            (self.cache.get_or_update(&key))
                .map(|(cached_val, result)| {
                    *val = cached_val.clone();
                    result
                })
                .unwrap_or_else(|cache| {
                    let result = (self.func)(Rc::make_mut(val));
                    &cache.push(key, (val.clone(), result)).1
                })
                .clone()
        }
    }
}

// --
// -- struct: RcCacheOwned
// --

/// Cache a function's output for Rc's with the same pointer value.
pub(crate) struct RcCacheOwned<T: Clone, U: Clone, F: FnMut(T) -> U> {
    cache: RcIndex<T, U>,
    func: F,
}

impl<T: Clone, U: Clone, F: FnMut(T) -> U> RcCacheOwned<T, U, F> {
    pub(crate) fn new(func: F) -> Self {
        Self { cache: Default::default(), func }
    }

    pub(crate) fn apply(&mut self, val: Rc<T>) -> U {
        let key = Rc::downgrade(&val);

        match Rc::try_unwrap(val) {
            Ok(val) => self.cache.pop(&key).unwrap_or_else(|| (self.func)(val)),
            Err(val) => (self.cache.get_or_update(&key))
                .unwrap_or_else(|index| {
                    let result = (self.func)(Rc::unwrap_or_clone(val));
                    index.push(key, result)
                })
                .clone(),
        }
    }
}

// --
// -- struct: RcIndex
// --

/// Map type that indexes other a weak Rc reference using a vec. Most operations are
/// O(self.index.len()) so this is not suitable for heavy computations.
struct RcIndex<T, U> {
    index: Vec<(Weak<T>, U)>,
}

impl<T, U> RcIndex<T, U> {
    /// Get the index of a value in current index.
    fn index(&self, key: &Weak<T>) -> Option<usize> {
        (self.index.iter())
            .enumerate()
            .rev()
            .find(|(_, (k, _))| k.ptr_eq(key))
            .map(|(idx, _)| idx)
    }

    /// Get a value from this index, if the key is not found a mutable reference to self is returned
    /// to allow updating the index.
    fn get_or_update<'a>(&'a mut self, key: &Weak<T>) -> Result<&'a U, &'a mut Self> {
        match self.index(key) {
            Some(idx) => Ok(&self.index[idx].1),
            None => Err(self),
        }
    }

    /// Get a value and remove it from the index.
    fn pop(&mut self, key: &Weak<T>) -> Option<U> {
        self.index
            .extract_if(.., |(k, _)| k.ptr_eq(key))
            .last()
            .map(|(_, val)| val)
    }

    /// Add a value to the index without checking if was already available.
    fn push(&mut self, key: Weak<T>, val: U) -> &U {
        &self.index.push_mut((key, val)).1
    }
}

impl<T, U> Default for RcIndex<T, U> {
    fn default() -> Self {
        Self { index: Vec::new() }
    }
}
