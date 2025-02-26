//! This module defines a data structure that allows assigning arbitrary values to arbitrary
//! subsets of an n-dimension space. It can then be used to extract maximally expanded
//! selectors of same value.
//!
//! This last problem is hard, so the implementation here only focuses on being convenient and
//! predictable. For example this research paper show that there is no known polytime approximation
//! for this problem in two dimensions and for boolean values :
//! https://dl.acm.org/doi/10.1145/73833.73871

use std::fmt::Debug;
use std::ops::Range;

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
// -- Paving
// --

/// Interface over a n-dim paving.
pub(crate) trait Paving: Clone + Default {
    type Selector;
    type Value: Clone + Default + Eq + Ord;
    fn set(&mut self, selector: &Self::Selector, val: &Self::Value);
    fn is_val(&self, selector: &Self::Selector, val: &Self::Value) -> bool;

    fn pop_filter(
        &mut self,
        filter: impl Fn(&Self::Value) -> bool,
    ) -> Option<(Self::Value, Self::Selector)>;
}

// --
// -- Cell
// --

/// Just a 0-dimension cell.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Cell<Val: Default + Eq + Ord> {
    inner: Val,
}

impl<Val: Clone + Default + Eq + Ord> Paving for Cell<Val> {
    type Selector = EmptyPavingSelector;
    type Value = Val;

    fn set(&mut self, _selector: &Self::Selector, val: &Val) {
        self.inner = val.clone();
    }

    fn is_val(&self, _selector: &Self::Selector, val: &Val) -> bool {
        self.inner == *val
    }

    fn pop_filter(
        &mut self,
        filter: impl Fn(&Self::Value) -> bool,
    ) -> Option<(Self::Value, Self::Selector)> {
        if filter(&self.inner) {
            Some((std::mem::take(&mut self.inner), EmptyPavingSelector))
        } else {
            None
        }
    }
}

// --
// -- Dim
// --

/// Add a dimension over a lower dimension paving.
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

impl<T: Clone + Ord + std::fmt::Debug, U: Paving + std::fmt::Debug> std::fmt::Debug for Dim<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.cols.is_empty() {
            f.debug_tuple("Dim::Empty").field(&self.cols).finish()?;
        }

        let mut fmt = f.debug_struct("Dim");

        for ((start, end), value) in (self.cuts.iter())
            .zip(self.cuts.iter().skip(1))
            .zip(&self.cols)
        {
            fmt.field(&format!("[{start:?}, {end:?}["), value);
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

impl<T: Clone + Ord, U: Paving> Paving for Dim<T, U> {
    type Selector = PavingSelector<T, U::Selector>;
    type Value = U::Value;

    fn set(&mut self, selector: &Self::Selector, val: &Self::Value) {
        let (ranges, selector_tail) = selector.unpack_front();

        for range in ranges {
            self.cut_at(range.start.clone());
            self.cut_at(range.end.clone());

            for (col_start, col_val) in self.cuts.iter().zip(&mut self.cols) {
                if *col_start >= range.start && *col_start < range.end {
                    col_val.set(selector_tail, val);
                }
            }
        }
    }

    fn is_val(&self, selector: &Self::Selector, val: &Self::Value) -> bool {
        let (ranges, selector_tail) = selector.unpack_front();

        for range in ranges {
            if range.start >= range.end {
                // Wrapping ranges are not supported.
                continue;
            }

            if self.cols.is_empty()
                || range.start < *self.cuts.first().unwrap()
                || range.end > *self.cuts.last().unwrap()
            {
                // There is either no columns either an overlap before the
                // first column or the last one. In these cases we just need
                // to ensure the requested value is the default.
                return *val == Self::Value::default();
            }

            for ((col_start, col_end), col_val) in self
                .cuts
                .iter()
                .zip(self.cuts.iter().skip(1))
                .zip(&self.cols)
            {
                // Column overlaps with the input range
                let col_overlaps = *col_start < range.end && *col_end > range.start;

                if col_overlaps && !col_val.is_val(selector_tail, val) {
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
}

// NOTE: this is heavily unoptimized, so we ensure that it is only used for tests
#[cfg(test)]
impl<T: Clone + Ord + std::fmt::Debug, U: Paving + PartialEq> PartialEq for Dim<T, U> {
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
