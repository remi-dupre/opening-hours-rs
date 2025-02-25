#![allow(clippy::single_range_in_vec_init)]
use crate::normalize::paving::*;

#[test]
fn test_dim2() {
    let grid_empty: Paving2D<i32, i32, bool> = Paving2D::default();

    //   0 1 2 3 4 5
    // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    // 3 ⋅ X X X X ⋅
    // 4 ⋅ X X X X ⋅
    // 5 ⋅ X X X X ⋅
    // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    let mut grid_1 = grid_empty.clone();

    grid_1.set(
        &EmptyPavingSelector
            .dim_front::<i32>([1..5])
            .dim_front::<i32>([3..6]),
        &true,
    );

    assert_ne!(grid_empty, grid_1);

    //   0 1 2 3 4 5
    // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    // 3 ⋅ A # # B ⋅
    // 4 ⋅ C C C C ⋅
    // 5 ⋅ C C C C ⋅
    // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    let mut grid_2 = grid_empty.clone();
    grid_2.set(
        &EmptyPavingSelector.dim_front([1..4]).dim_front([3..4]),
        &true,
    ); // A & #
    grid_2.set(
        &EmptyPavingSelector.dim_front([2..5]).dim_front([3..4]),
        &true,
    ); // B & #
    grid_2.set(
        &EmptyPavingSelector.dim_front([1..5]).dim_front([4..6]),
        &true,
    ); // C
    assert_eq!(grid_1, grid_2);
}

#[test]
fn test_pop_trivial() {
    let grid_empty = Paving2D::default();

    //   0 1 2 3 4 5
    // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    // 3 ⋅ A # # B ⋅
    // 4 ⋅ C C C C ⋅
    // 5 ⋅ C C C C ⋅
    // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    let mut grid = grid_empty.clone();
    grid.set(
        &EmptyPavingSelector.dim_front([1..4]).dim_front([3..4]),
        &true,
    ); // A & #
    grid.set(
        &EmptyPavingSelector.dim_front([2..5]).dim_front([3..4]),
        &true,
    ); // B & #
    grid.set(
        &EmptyPavingSelector.dim_front([1..5]).dim_front([4..6]),
        &true,
    ); // C

    assert_eq!(
        grid.pop_filter(|x| *x).unwrap().1,
        EmptyPavingSelector.dim_front([1..5]).dim_front([3..6]),
    );

    assert_eq!(grid, grid_empty);
    assert_eq!(grid.pop_filter(|x| *x), None);
}

#[test]
fn test_pop_disjoint() {
    let grid_empty = Paving2D::default();

    //   0 1 2 3 4 5 6 7
    // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    // 3 ⋅ A ⋅ ⋅ B B B ⋅
    // 4 ⋅ A ⋅ ⋅ B B B ⋅
    // 5 ⋅ A ⋅ ⋅ B B B ⋅
    // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    let mut grid = grid_empty.clone();

    grid.set(
        &EmptyPavingSelector
            .dim_front([1..2, 4..7])
            .dim_front([3..6]),
        &true,
    );

    assert_eq!(
        grid.pop_filter(|x| *x).unwrap().1,
        EmptyPavingSelector
            .dim_front([1..2, 4..7])
            .dim_front([3..6]),
    );

    assert_eq!(grid, grid_empty);
    assert_eq!(grid.pop_filter(|x| *x), None);
}

#[test]
fn test_debug() {
    //   0 1 2 3 4 5 6 7
    // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    // 3 ⋅ A ⋅ ⋅ B B B ⋅
    // 4 ⋅ A ⋅ ⋅ B B B ⋅
    // 5 ⋅ A ⋅ ⋅ B B B ⋅
    // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    let mut grid = Paving2D::default();

    grid.set(
        &EmptyPavingSelector
            .dim_front([1..2, 4..7])
            .dim_front([3..6]),
        &true,
    );

    assert_eq!(format!("{grid:?}"), "Dim { [3, 6[: Dim { [1, 2[: Cell { inner: true }, [2, 4[: Cell { inner: false }, [4, 7[: Cell { inner: true } } }")
}
