use std::fmt::Debug;
use std::ops::Range;

pub(crate) type Paving1D<T> = Dim<T, Cell>;
pub(crate) type Paving2D<T, U> = Dim<T, Paving1D<U>>;
pub(crate) type Paving3D<T, U, V> = Dim<T, Paving2D<U, V>>;
pub(crate) type Paving4D<T, U, V, W> = Dim<T, Paving3D<U, V, W>>;
pub(crate) type Paving5D<T, U, V, W, X> = Dim<T, Paving4D<U, V, W, X>>;

pub(crate) type SelectorEmpty = PavingSelector<(), ()>;
pub(crate) type Selector1D<T> = PavingSelector<T, SelectorEmpty>;
pub(crate) type Selector2D<T, U> = PavingSelector<T, Selector1D<U>>;
pub(crate) type Selector3D<T, U, V> = PavingSelector<T, Selector2D<U, V>>;
pub(crate) type Selector4D<T, U, V, W> = PavingSelector<T, Selector3D<U, V, W>>;
pub(crate) type Selector5D<T, U, V, W, X> = PavingSelector<T, Selector4D<U, V, W, X>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PavingSelector<T, U> {
    Empty,
    Dim { range: Vec<Range<T>>, tail: U },
}

impl PavingSelector<(), ()> {
    pub(crate) fn empty() -> PavingSelector<(), ()> {
        PavingSelector::<(), ()>::Empty
    }
}

impl<T, U> PavingSelector<T, U> {
    pub(crate) fn dim<K>(
        self,
        range: impl Into<Vec<Range<K>>>,
    ) -> PavingSelector<K, PavingSelector<T, U>> {
        PavingSelector::Dim { range: range.into(), tail: self }
    }

    pub(crate) fn unpack(&self) -> (&[Range<T>], &U) {
        let Self::Dim { range, tail } = &self else {
            panic!("tried to unpack empty selector");
        };

        (range, tail)
    }
}

pub(crate) trait Paving: Clone + Default {
    type Selector;
    fn union_with(&mut self, other: Self);
    fn set(&mut self, selector: &Self::Selector, val: bool);
    fn is_val(&self, selector: &Self::Selector, val: bool) -> bool;
    fn pop_selector(&mut self) -> Option<Self::Selector>;
}

// Just a 0-dimension cell that is either filled or empty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Cell {
    filled: bool,
}

impl Paving for Cell {
    type Selector = PavingSelector<(), ()>;

    fn union_with(&mut self, other: Self) {
        self.filled |= other.filled;
    }

    fn set(&mut self, _selector: &Self::Selector, val: bool) {
        self.filled = val;
    }

    fn pop_selector(&mut self) -> Option<Self::Selector> {
        if self.filled {
            Some(PavingSelector::empty())
        } else {
            None
        }
    }

    fn is_val(&self, _selector: &Self::Selector, val: bool) -> bool {
        self.filled == val
    }
}

// Add a dimension over a lower dimension paving.
// TODO: when some benchmark is implemented, check if a dequeue is better.
#[derive(Clone, Debug)]
pub(crate) struct Dim<T: Clone + Ord, U: Paving> {
    cuts: Vec<T>, // ordered
    cols: Vec<U>, // one less elements than cuts
}

impl<T: Clone + Ord, U: Paving> Default for Dim<T, U> {
    fn default() -> Self {
        Self { cuts: Vec::new(), cols: Vec::new() }
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
            // Added the cut after the end
            self.cols.push(U::default())
        } else if insert_pos == 0 {
            // Added the cut before the start
            self.cols.insert(0, U::default())
        } else {
            let cut_fill = self.cols[insert_pos - 1].clone();
            self.cols.insert(insert_pos, cut_fill);
        }
    }
}

impl<T: Clone + Ord, U: Default + Paving> Paving for Dim<T, U> {
    type Selector = PavingSelector<T, U::Selector>;

    fn union_with(&mut self, mut other: Self) {
        // First, ensure both parts have the same cuts
        for cut in &self.cuts {
            other.cut_at(cut.clone());
        }

        for cut in other.cuts {
            self.cut_at(cut);
        }

        // Now that the dimensions are the same, merge all "columns"
        for (col_self, col_other) in self.cols.iter_mut().zip(other.cols.into_iter()) {
            col_self.union_with(col_other);
        }
    }

    fn set(&mut self, selector: &Self::Selector, val: bool) {
        let (ranges, selector_tail) = selector.unpack();

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

    fn is_val(&self, selector: &Self::Selector, val: bool) -> bool {
        let (ranges, selector_tail) = selector.unpack();

        for range in ranges {
            if range.start < range.end && self.cols.is_empty() {
                return !val;
            }

            for ((col_start, col_end), col_val) in self
                .cuts
                .iter()
                .zip(self.cuts.iter().skip(1))
                .zip(&self.cols)
            {
                // TODO: don't I miss something?
                if *col_start < range.end
                    && *col_end > range.start
                    && !col_val.is_val(selector_tail, val)
                {
                    return false;
                }
            }
        }

        true
    }

    fn pop_selector(&mut self) -> Option<Self::Selector> {
        let (mut start_idx, selector_tail) = self
            .cols
            .iter_mut()
            .enumerate()
            .find_map(|(idx, col)| Some((idx, col.pop_selector()?)))?;

        let mut end_idx = start_idx + 1;
        let mut selector_range = Vec::new();

        while end_idx < self.cols.len() {
            if self.cols[end_idx].is_val(&selector_tail, true) {
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

        let selector = PavingSelector::Dim { range: selector_range, tail: selector_tail };
        self.set(&selector, false);
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
