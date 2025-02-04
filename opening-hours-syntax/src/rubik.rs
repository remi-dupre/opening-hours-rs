#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PavingSelector<T, U> {
    Empty,
    Dim { min: T, max: T, tail: U },
}

impl PavingSelector<(), ()> {
    pub(crate) fn empty() -> PavingSelector<(), ()> {
        PavingSelector::Empty
    }
}

impl<T, U> PavingSelector<T, U> {
    pub(crate) fn dim<K>(self, min: K, max: K) -> PavingSelector<K, PavingSelector<T, U>> {
        PavingSelector::Dim { min, max, tail: self }
    }

    pub(crate) fn unpack(&self) -> (&T, &T, &U) {
        let Self::Dim { min, max, tail } = &self else {
            panic!("tried to unpack empty selector");
        };

        (min, max, tail)
    }
}

pub(crate) trait Paving: Clone {
    type Selector;
    fn union_with(&mut self, other: Self);
    fn set(&mut self, selector: &Self::Selector, val: bool);
    fn is_val(&self, selector: &Self::Selector, val: bool) -> bool;
    fn pop_selector(&mut self) -> Option<Self::Selector>;

    fn dim<T: Clone + Ord>(self, min: T, max: T) -> Dim<T, Self> {
        Dim { cuts: vec![min, max], cols: vec![self] }
    }
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

// Add a dimension over a lower dimension paving
#[derive(Clone, Debug)]
pub(crate) struct Dim<T: Clone + Ord, U: Paving> {
    cuts: Vec<T>, // ordered and at least 2 elements
    cols: Vec<U>, // on less elements than cuts
}

impl<T: Clone + Ord, U: Paving> Dim<T, U> {
    fn cut_at(&mut self, val: T) {
        let Err(insert_pos) = self.cuts.binary_search(&val) else {
            // Already cut at given position
            return;
        };

        let cut_fill = self.cols[insert_pos - 1].clone();
        self.cuts.insert(insert_pos, val);
        self.cols.insert(insert_pos, cut_fill);
    }
}

impl<T: Clone + Ord, U: Paving> Paving for Dim<T, U> {
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
        let (min, max, selector_tail) = selector.unpack();
        self.cut_at(min.clone());
        self.cut_at(max.clone());

        for (col_start, col_val) in self.cuts.iter().zip(&mut self.cols) {
            if col_start >= min && col_start < max {
                col_val.set(selector_tail, val);
            }
        }
    }

    fn is_val(&self, selector: &Self::Selector, val: bool) -> bool {
        let (min, max, selector_tail) = selector.unpack();

        for ((col_start, col_end), col_val) in self.cuts.iter().zip(&self.cuts[1..]).zip(&self.cols)
        {
            // TODO: don't I miss something
            if col_start < max && col_end > min && !col_val.is_val(selector_tail, val) {
                return false;
            }
        }

        true
    }

    fn pop_selector(&mut self) -> Option<Self::Selector> {
        let (start_idx, selector_tail) = self
            .cols
            .iter_mut()
            .enumerate()
            .find_map(|(idx, col)| Some((idx, col.pop_selector()?)))?;

        let mut end_idx = start_idx + 1;

        while end_idx < self.cols.len() && self.cols[end_idx].is_val(&selector_tail, true) {
            end_idx += 1;
        }

        let selector = PavingSelector::Dim {
            min: self.cuts[start_idx].clone(),
            max: self.cuts[end_idx].clone(),
            tail: selector_tail,
        };

        self.set(&selector, false);
        Some(selector)
    }
}

impl<T: Clone + Ord + std::fmt::Debug, U: Paving + PartialEq + std::fmt::Debug> PartialEq
    for Dim<T, U>
{
    fn eq(&self, other: &Self) -> bool {
        for ((s_min, s_max), s_col) in self.cuts.iter().zip(&self.cuts[1..]).zip(&self.cols) {
            for ((o_min, o_max), o_col) in other.cuts.iter().zip(&other.cuts[1..]).zip(&other.cols)
            {
                if o_min >= s_max || s_min >= o_max {
                    // no overlap
                    continue;
                }

                if s_col != o_col {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_dim2() {
        let grid_empty = Cell::default().dim(0, 6).dim(0, 6);

        //   0 1 2 3 4 5
        // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
        // 3 ⋅ X X X X ⋅
        // 4 ⋅ X X X X ⋅
        // 5 ⋅ X X X X ⋅
        // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
        let mut grid_1 = grid_empty.clone();
        grid_1.set(&PavingSelector::empty().dim(1, 5).dim(3, 6), true);
        assert_ne!(grid_empty, grid_1);

        //   0 1 2 3 4 5
        // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
        // 3 ⋅ A # # B ⋅
        // 4 ⋅ C C C C ⋅
        // 5 ⋅ C C C C ⋅
        // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
        let mut grid_2 = grid_empty.clone();
        grid_2.set(&PavingSelector::empty().dim(1, 4).dim(3, 4), true); // A & #
        grid_2.set(&PavingSelector::empty().dim(2, 5).dim(3, 4), true); // B & #
        grid_2.set(&PavingSelector::empty().dim(1, 5).dim(4, 6), true); // C
        assert_eq!(grid_1, grid_2);
    }

    #[test]
    fn test_pop_trivial() {
        let grid_empty = Cell::default().dim(0, 6).dim(0, 6);

        //   0 1 2 3 4 5
        // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
        // 3 ⋅ A # # B ⋅
        // 4 ⋅ C C C C ⋅
        // 5 ⋅ C C C C ⋅
        // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
        let mut grid = grid_empty.clone();
        grid.set(&PavingSelector::empty().dim(1, 4).dim(3, 4), true); // A & #
        grid.set(&PavingSelector::empty().dim(2, 5).dim(3, 4), true); // B & #
        grid.set(&PavingSelector::empty().dim(1, 5).dim(4, 6), true); // C

        assert_eq!(
            grid.pop_selector().unwrap(),
            PavingSelector::empty().dim(1, 5).dim(3, 6),
        );

        assert_eq!(grid, grid_empty);
        assert_eq!(grid.pop_selector(), None);
    }
}
