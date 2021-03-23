use std::borrow::Borrow;
use std::cmp::Ordering;
use std::convert::From;
use std::fmt;
use std::ops::Deref;

/// A wrapper arround a [`Vec`] that is always sorted and with values repeating
/// at most once.
///
/// ```
/// use opening_hours_syntax::sorted_vec::UniqueSortedVec;
///
/// let sorted: UniqueSortedVec<_> = vec![2, 1, 3, 5, 3].into();
/// assert_eq!(sorted.as_slice(), &[1, 2, 3, 5]);
/// ```
#[repr(transparent)]
#[derive(Default)]
pub struct UniqueSortedVec<T>(Vec<T>);

impl<T> UniqueSortedVec<T> {
    /// Create a new empty instance.
    #[inline]
    pub const fn new() -> Self {
        Self(Vec::new())
    }
}

impl<T: Ord> UniqueSortedVec<T> {
    /// Build a new [`UniqueSortedVec`] with borrowed content. The order is
    /// assumed to be equivalent for borrowed content.
    ///
    /// ```
    /// use opening_hours_syntax::sorted_vec::UniqueSortedVec;
    ///
    /// let sorted: UniqueSortedVec<_> = vec!["Hello".to_string(), "Anaïs".to_string()].into();
    /// let sorted_ref: UniqueSortedVec<&str> = sorted.to_ref();
    /// assert_eq!(sorted_ref.as_slice(), &["Anaïs", "Hello"]);
    /// ```
    #[inline]
    pub fn to_ref<U: Ord + ?Sized>(&self) -> UniqueSortedVec<&U>
    where
        T: Borrow<U>,
    {
        UniqueSortedVec(self.0.iter().map(Borrow::borrow).collect())
    }

    /// Merge values of two [`UniqueSortedVec`] while preserving the invariants.
    ///
    /// ```
    /// use opening_hours_syntax::sorted_vec::UniqueSortedVec;
    ///
    /// let sorted_1: UniqueSortedVec<_> = vec![1, 2, 3].into();
    /// let sorted_2: UniqueSortedVec<_> = vec![0, 3, 4].into();
    /// assert_eq!(sorted_1.union(sorted_2).as_slice(), &[0, 1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn union(mut self, mut other: Self) -> Self {
        match (self.as_slice(), other.as_slice()) {
            (_, []) => self,
            ([], _) => other,
            ([.., tail_x], [.., tail_y]) => {
                let last = match tail_x.cmp(&tail_y) {
                    Ordering::Greater => self.0.pop().unwrap(),
                    Ordering::Less => other.0.pop().unwrap(),
                    Ordering::Equal => {
                        other.0.pop().unwrap();
                        self.0.pop().unwrap()
                    }
                };

                let mut new_head = self.union(other);
                new_head.0.push(last);
                new_head
            }
        }
    }

    /// Returns true if the slice contains an element with the given value.
    ///
    /// ```
    /// use opening_hours_syntax::sorted_vec::UniqueSortedVec;
    ///
    /// let sorted: UniqueSortedVec<_> = vec![10, 40, 30].into();
    /// assert!(sorted.contains(&30));
    /// assert!(!sorted.contains(&50));
    /// ```
    #[inline]
    pub fn contains(&self, x: &T) -> bool {
        self.0.binary_search(x).is_ok()
    }

    /// Search for the given value in the slice return a reference to it or the
    /// next greater value if it is not found.
    ///
    /// ```
    /// use opening_hours_syntax::sorted_vec::UniqueSortedVec;
    ///
    /// let sorted: UniqueSortedVec<_> = vec![10, 40, 30].into();
    /// assert_eq!(sorted.find_first_following(&30), Some(&30));
    /// assert_eq!(sorted.find_first_following(&31), Some(&40));
    /// assert_eq!(sorted.find_first_following(&50), None);
    /// ```
    #[inline]
    pub fn find_first_following(&self, x: &T) -> Option<&T> {
        match self.0.binary_search(x) {
            Ok(i) | Err(i) => self.0.get(i),
        }
    }
}

impl<T: Ord> From<Vec<T>> for UniqueSortedVec<T> {
    #[inline]
    fn from(mut vec: Vec<T>) -> Self {
        vec.sort_unstable();
        vec.dedup();
        Self(vec)
    }
}

impl<T: Ord> Into<Vec<T>> for UniqueSortedVec<T> {
    #[inline]
    fn into(self) -> Vec<T> {
        self.0
    }
}

// ---
// --- Dummy trait implementations.
// ---

impl<T: Ord + Clone> Clone for UniqueSortedVec<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Ord + fmt::Debug> fmt::Debug for UniqueSortedVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UniqueSortedVec({:?})", self.0)
    }
}

impl<T: Ord> Deref for UniqueSortedVec<T> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Ord + Eq> Eq for UniqueSortedVec<T> {}

impl<T: Ord + PartialEq> PartialEq for UniqueSortedVec<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
