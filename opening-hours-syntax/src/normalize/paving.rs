//! This module defines a data structure that allows assigning arbitrary values to arbitrary
//! subsets of an n-dimension space. It can then be used to extract maximally expanded
//! selectors of same value.
//!
//! This last problem is hard, so the implementation here only focuses on being convenient and
//! predictable. For example this research paper show that there is no known polytime approximation
//! for this problem in two dimensions and for boolean values :
//! https://dl.acm.org/doi/10.1145/73833.73871

use alloc::vec::Vec;
use core::fmt::Debug;
use core::ops::Range;

pub(crate) type Paving1D<T, Val> = Dim<T, Cell<Val>>;
pub(crate) type Paving2D<T, U, Val> = Dim<T, Paving1D<U, Val>>;
pub(crate) type Paving3D<T, U, V, Val> = Dim<T, Paving2D<U, V, Val>>;
pub(crate) type Paving4D<T, U, V, W, Val> = Dim<T, Paving3D<U, V, W, Val>>;
pub(crate) type Paving5D<T, U, V, W, X, Val> = Dim<T, Paving4D<U, V, W, X, Val>>;

pub(crate) type Selector1D<T> = PavingSelector<T, EmptyPavingSelector>;
pub(crate) type Selector2D<T, U> = PavingSelector<T, Selector1D<U>>;
pub(crate) type Selector3D<T, U, V> = PavingSelector<T, Selector2D<U, V>>;
pub(crate) type Selector4D<T, U, V, W> = PavingSelector<T, Selector3D<U, V, W>>;
pub(crate) type Selector5D<T, U, V, W, X> = PavingSelector<T, Selector4D<U, V, W, X>>;

// --
// -- EmptyPavingSelector
// --

/// A selector for paving of zero dimensions. This is a convenience type
/// intented to be expanded with more dimensions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EmptyPavingSelector;

impl EmptyPavingSelector {
    pub(crate) fn dim_front<T>(self, range: impl Into<Vec<Range<T>>>) -> PavingSelector<T, Self> {
        PavingSelector { range: range.into(), tail: self }
    }
}

// --
// -- PavingSelector
// --

/// A selector for a paving for at least one dimension. Recursively built for
/// each dimensions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PavingSelector<T, U> {
    range: Vec<Range<T>>,
    tail: U,
}

impl<T, U> PavingSelector<T, U> {
    pub(crate) fn dim_front<V>(self, range: impl Into<Vec<Range<V>>>) -> PavingSelector<V, Self> {
        PavingSelector { range: range.into(), tail: self }
    }

    pub(crate) fn unpack_front(&self) -> (&[Range<T>], &U) {
        (&self.range, &self.tail)
    }

    pub(crate) fn into_unpack_front(self) -> (Vec<Range<T>>, U) {
        (self.range, self.tail)
    }
}

// --
// -- Compression
// --

pub(crate) trait SelectorCompression: Sized {
    /// Attempt to merge consecutive intervals from the front dimension as
    /// long as this preserves the input predicate.
    fn fill_holes_front(&mut self, _predicate: impl FnMut(&Self) -> bool);

    /// Recursively merge consecutive intervals from tail dimensions as
    /// long as this preserves the input predicate.
    fn fill_holes_back(&mut self, _predicate: impl FnMut(&Self) -> bool);

    /// Recursively merge intervals as long as it preserves the input
    /// predicate.
    fn fill_holes(&mut self, mut predicate: impl FnMut(&Self) -> bool) {
        self.fill_holes_front(&mut predicate);
        self.fill_holes_back(predicate);
    }
}

impl SelectorCompression for EmptyPavingSelector {
    fn fill_holes_front(&mut self, _predicate: impl FnMut(&Self) -> bool) {}
    fn fill_holes_back(&mut self, _predicate: impl FnMut(&Self) -> bool) {}
}

impl<T: Clone + Debug, U: Clone + Debug + SelectorCompression> SelectorCompression
    for PavingSelector<T, U>
{
    fn fill_holes_front(&mut self, mut predicate: impl FnMut(&Self) -> bool) {
        for idx in (0..self.range.len() - 1).rev() {
            // Backup the two intervals we attempt to merge
            let rg_left = self.range.remove(idx);
            let rg_right = self.range[idx].clone();

            // Apply compression in-place
            self.range[idx].start = rg_left.start.clone();

            // If the predicate is true with current compression, keep in-place
            // modifications. Otherwise restore previous value.
            if !predicate(self) {
                self.range[idx] = rg_right;
                self.range.insert(idx, rg_left);
            }
        }
    }

    fn fill_holes_back(&mut self, mut predicate: impl FnMut(&Self) -> bool) {
        self.tail.fill_holes(|tail_attempt| {
            let attempt = PavingSelector {
                range: self.range.clone(),
                tail: tail_attempt.clone(),
            };

            predicate(&attempt)
        })
    }
}

// --
// -- Unpack from the back
// --

/// Allows to pop selector from the back with no clone. Note that it is still
/// slower than popping from the front.
pub(crate) trait UnpackFromBack {
    type Head;
    type BackVal;

    /// Pop back value
    fn into_unpack_back(self) -> (Self::Head, Self::BackVal);

    /// Modify back value in place
    fn substitute_back(&mut self, ranges: impl Into<Self::BackVal>);
}

impl<T> UnpackFromBack for PavingSelector<T, EmptyPavingSelector> {
    type Head = EmptyPavingSelector;
    type BackVal = Vec<Range<T>>;

    fn into_unpack_back(self) -> (Self::Head, Self::BackVal) {
        (EmptyPavingSelector, self.range)
    }

    fn substitute_back(&mut self, ranges: impl Into<Self::BackVal>) {
        self.range = ranges.into();
    }
}

impl<T, U: UnpackFromBack> UnpackFromBack for PavingSelector<T, U> {
    type Head = PavingSelector<T, U::Head>;
    type BackVal = U::BackVal;

    fn into_unpack_back(self) -> (Self::Head, Self::BackVal) {
        let (new_tail, back_val) = self.tail.into_unpack_back();
        let head = PavingSelector { range: self.range, tail: new_tail };
        (head, back_val)
    }

    fn substitute_back(&mut self, ranges: impl Into<Self::BackVal>) {
        self.tail.substitute_back(ranges)
    }
}

// --
// -- Paving
// --

/// Interface over a n-dim paving.
pub(crate) trait Paving: Clone + Debug + Default {
    type Selector: Debug;
    type Value: Clone + Default + Eq;
    type Map<X: Clone + Debug + Default + Eq>: Paving;

    fn update(&mut self, selector: &Self::Selector, operation: impl FnMut(&mut Self::Value));

    fn map<X: Clone + Debug + Default + Eq>(
        self,
        map: impl FnMut(Self::Value) -> X,
    ) -> Self::Map<X>;

    fn check_predicate(
        &self,
        selector: &Self::Selector,
        predicate: impl FnMut(&Self::Value) -> bool,
    ) -> bool;

    fn pop_filter(
        &mut self,
        filter: impl Fn(&Self::Value) -> bool,
    ) -> Option<(Self::Value, Self::Selector)>;

    fn is_val(&self, selector: &Self::Selector, val: &Self::Value) -> bool {
        self.check_predicate(selector, move |part| part == val)
    }

    fn set(&mut self, selector: &Self::Selector, val: &Self::Value) {
        self.update(selector, |inner| *inner = val.clone());
    }
}

// --
// -- Cell
// --

/// Just a 0-dimension cell.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Cell<Val: Default + Eq> {
    inner: Val,
}

impl<Val: Clone + Debug + Default + Eq> Paving for Cell<Val> {
    type Selector = EmptyPavingSelector;
    type Value = Val;
    type Map<X: Clone + Debug + Default + Eq> = Cell<X>;

    fn update(&mut self, _selector: &Self::Selector, mut operation: impl FnMut(&mut Self::Value)) {
        operation(&mut self.inner)
    }

    fn check_predicate(
        &self,
        _selector: &Self::Selector,
        mut predicate: impl FnMut(&Self::Value) -> bool,
    ) -> bool {
        predicate(&self.inner)
    }

    fn pop_filter(
        &mut self,
        filter: impl Fn(&Self::Value) -> bool,
    ) -> Option<(Self::Value, Self::Selector)> {
        if filter(&self.inner) {
            Some((core::mem::take(&mut self.inner), EmptyPavingSelector))
        } else {
            None
        }
    }

    fn map<X: Clone + Debug + Default + Eq>(
        self,
        mut map: impl FnMut(Self::Value) -> X,
    ) -> Self::Map<X> {
        Cell { inner: map(self.inner) }
    }
}

// --
// -- Dim
// --

/// Add a dimension over a lower dimension paving. It consists of a sequence
/// of n cuts which delimits n-1 cols, as follows:
///
///   cut 0    cut 1    cut 2 ... cut n
///     |  col1  |  col2  |   ...   |
#[derive(Clone)]
pub(crate) struct Dim<T: Clone + Ord, U: Paving> {
    cuts: Vec<T>, // ordered
    cols: Vec<U>, // one less elements than cuts
}

impl<T: Clone + Ord, U: Paving> Default for Dim<T, U> {
    fn default() -> Self {
        Self { cuts: Vec::new(), cols: Vec::new() }
    }
}

impl<T: Clone + Ord + Debug, U: Paving + Debug> Debug for Dim<T, U> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.cols.is_empty() {
            f.debug_tuple("Dim::Empty").field(&self.cols).finish()?;
        }

        let mut fmt = f.debug_struct("Dim");

        for ((start, end), value) in (self.cuts.iter())
            .zip(self.cuts.iter().skip(1))
            .zip(&self.cols)
        {
            fmt.field(&format!("{start:?}..{end:?}"), value);
        }

        fmt.finish()
    }
}

impl<T: Clone + Ord, U: Paving> Dim<T, U> {
    fn cut_at(&mut self, val: T) {
        let Err(insert_pos) = self.cuts.binary_search(&val) else {
            // Already cut at given position
            return;
        };

        self.cuts.insert(insert_pos, val);
        debug_assert!(self.cuts.is_sorted());

        if self.cuts.len() == 1 {
            // No interval created yet
        } else if self.cuts.len() == 2 {
            // First interval
            self.cols.push(U::default())
        } else if insert_pos == self.cuts.len() - 1 {
            // Added the cut at the end
            self.cols.push(U::default())
        } else if insert_pos == 0 {
            // Added the cut at the start
            self.cols.insert(0, U::default())
        } else {
            let cut_fill = self.cols[insert_pos - 1].clone();
            self.cols.insert(insert_pos, cut_fill);
        }
    }
}

impl<T: Clone + Debug + Ord, U: Debug + Paving> Paving for Dim<T, U> {
    type Selector = PavingSelector<T, U::Selector>;
    type Value = U::Value;
    type Map<X: Clone + Debug + Default + Eq> = Dim<T, U::Map<X>>;

    fn update(&mut self, selector: &Self::Selector, mut operation: impl FnMut(&mut Self::Value)) {
        let (ranges, selector_tail) = selector.unpack_front();

        for range in ranges {
            self.cut_at(range.start.clone());
            self.cut_at(range.end.clone());

            for (col_start, col_val) in self.cuts.iter().zip(&mut self.cols) {
                if *col_start >= range.start && *col_start < range.end {
                    col_val.update(selector_tail, &mut operation);
                }
            }
        }
    }

    /// Check if the *full* range covered by `selector` is set and equals to `val`.
    fn check_predicate(
        &self,
        selector: &Self::Selector,
        mut predicate: impl FnMut(&Self::Value) -> bool,
    ) -> bool {
        let (ranges, selector_tail) = selector.unpack_front();

        for range in ranges {
            // Wrapping ranges are not supported : inverted bounds are
            // considered an empty interval.
            if range.start >= range.end {
                continue;
            }

            // Check if part of the selector covers a part that is outside of
            // the explicitly set values for this paving.
            let partialy_outside_bounds = self.cols.is_empty()
                || range.start
                    < *(self.cuts.first()).expect("there is always on more cuts than columns")
                || range.end
                    > *(self.cuts.last()).expect("there is always on more cuts than columns");

            // If part of the selector is outside of explicitly set values, the
            // expected value must be the default.
            if partialy_outside_bounds && !predicate(&Self::Value::default()) {
                return false;
            }

            // Check value of overlapping columns
            for ((col_start, col_end), col_val) in (self.cuts.iter())
                .zip(self.cuts.iter().skip(1))
                .zip(&self.cols)
            {
                // Column overlaps with the input range
                let col_overlaps = *col_start < range.end && *col_end > range.start;

                if col_overlaps && !col_val.check_predicate(selector_tail, &mut predicate) {
                    return false;
                }
            }
        }

        true
    }

    fn pop_filter(
        &mut self,
        filter: impl Fn(&Self::Value) -> bool,
    ) -> Option<(Self::Value, Self::Selector)> {
        let (mut start_idx, (target_value, selector_tail)) = self
            .cols
            .iter_mut()
            .enumerate()
            .find_map(|(idx, col)| Some((idx, col.pop_filter(&filter)?)))?;

        let mut end_idx = start_idx + 1;
        let mut selector_range = Vec::new();

        while end_idx < self.cols.len() {
            if self.cols[end_idx].is_val(&selector_tail, &target_value) {
                end_idx += 1;
                continue;
            }

            if start_idx < end_idx {
                selector_range.push(self.cuts[start_idx].clone()..self.cuts[end_idx].clone());
            }

            end_idx += 1;
            start_idx = end_idx;
        }

        if start_idx < end_idx {
            selector_range.push(self.cuts[start_idx].clone()..self.cuts[end_idx].clone());
        }

        let selector = PavingSelector { range: selector_range, tail: selector_tail };
        self.set(&selector, &Self::Value::default());
        Some((target_value, selector))
    }

    fn map<X: Clone + Debug + Default + Eq>(
        self,
        mut map: impl FnMut(Self::Value) -> X,
    ) -> Self::Map<X> {
        Dim {
            cuts: self.cuts,
            cols: self.cols.into_iter().map(|col| col.map(&mut map)).collect(),
        }
    }
}

// NOTE: this is heavily unoptimized, so we ensure that it is only used for tests
#[cfg(test)]
impl<T: Clone + Ord + Debug, U: Paving + PartialEq> PartialEq for Dim<T, U> {
    fn eq(&self, other: &Self) -> bool {
        let mut self_cpy = self.clone();
        let mut other_cpy = other.clone();

        for cut in &self_cpy.cuts {
            other_cpy.cut_at(cut.clone());
        }

        for cut in &other_cpy.cuts {
            self_cpy.cut_at(cut.clone());
        }

        for (col_self, col_other) in self_cpy.cols.into_iter().zip(other_cpy.cols) {
            if col_self != col_other {
                return false;
            }
        }

        true
    }
}
