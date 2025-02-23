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
    pub(crate) fn dim<T>(self, range: impl Into<Vec<Range<T>>>) -> PavingSelector<T, Self> {
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

/// A trait that helps with accessing a selector from the back.
pub(crate) trait DimFromBack {
    /// The type of selector resulting from pushing a dimension `U` to the back.
    type PushedBack<U>;

    /// The type of selector that remains after popping a dimension from the back.
    type PoppedBack;

    /// The type of the dimension from the back.
    type BackType;

    fn dim_back<U>(self, range: impl Into<Vec<Range<U>>>) -> Self::PushedBack<U>;
    fn into_unpack_back(self) -> (Vec<Range<Self::BackType>>, Self::PoppedBack);
}

impl<X> DimFromBack for PavingSelector<X, EmptyPavingSelector> {
    type PushedBack<U> = PavingSelector<X, PavingSelector<U, EmptyPavingSelector>>;
    type PoppedBack = EmptyPavingSelector;
    type BackType = X;

    fn dim_back<U>(self, range: impl Into<Vec<Range<U>>>) -> Self::PushedBack<U> {
        EmptyPavingSelector.dim(range).dim_front(self.range)
    }

    fn into_unpack_back(self) -> (Vec<Range<Self::BackType>>, Self::PoppedBack) {
        (self.range, EmptyPavingSelector)
    }
}

impl<X, Y: DimFromBack> DimFromBack for PavingSelector<X, Y> {
    type PushedBack<U> = PavingSelector<X, Y::PushedBack<U>>;
    type PoppedBack = PavingSelector<X, Y::PoppedBack>;
    type BackType = Y::BackType;

    fn dim_back<U>(self, range: impl Into<Vec<Range<U>>>) -> Self::PushedBack<U> {
        PavingSelector { range: self.range, tail: self.tail.dim_back(range) }
    }

    fn into_unpack_back(self) -> (Vec<Range<Self::BackType>>, Self::PoppedBack) {
        let (unpacked, tail) = self.tail.into_unpack_back();
        (unpacked, PavingSelector { range: self.range, tail })
    }
}

// --
// -- Paving
// --

/// Interface over a n-dim paving.
pub(crate) trait Paving: Clone + Default {
    type Selector;
    type Value: Copy + Default + Eq + Ord;
    fn set(&mut self, selector: &Self::Selector, val: Self::Value);
    fn is_val(&self, selector: &Self::Selector, val: Self::Value) -> bool;
    fn pop_selector(&mut self, target_value: Self::Value) -> Option<Self::Selector>;
}

// --
// -- Cell
// --

/// Just a 0-dimension cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Cell<Val: Default + Eq + Ord> {
    inner: Val,
}

impl<Val: Copy + Default + Eq + Ord> Paving for Cell<Val> {
    type Selector = EmptyPavingSelector;
    type Value = Val;

    fn set(&mut self, _selector: &Self::Selector, val: Val) {
        self.inner = val;
    }

    fn is_val(&self, _selector: &Self::Selector, val: Val) -> bool {
        self.inner == val
    }

    fn pop_selector(&mut self, target_value: Val) -> Option<Self::Selector> {
        if self.inner == target_value {
            Some(EmptyPavingSelector)
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

    fn set(&mut self, selector: &Self::Selector, val: Self::Value) {
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

    fn is_val(&self, selector: &Self::Selector, val: Self::Value) -> bool {
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
                return val == Self::Value::default();
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

    fn pop_selector(&mut self, target_value: Self::Value) -> Option<Self::Selector> {
        let (mut start_idx, selector_tail) = self
            .cols
            .iter_mut()
            .enumerate()
            .find_map(|(idx, col)| Some((idx, col.pop_selector(target_value)?)))?;

        let mut end_idx = start_idx + 1;
        let mut selector_range = Vec::new();

        while end_idx < self.cols.len() {
            if self.cols[end_idx].is_val(&selector_tail, target_value) {
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
        self.set(&selector, Self::Value::default());
        Some(selector)
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
