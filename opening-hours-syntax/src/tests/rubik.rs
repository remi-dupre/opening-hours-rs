use crate::rubik::*;

#[test]
fn test_dim2() {
    let grid_empty: Paving2D<i32, i32> = Paving2D::default();

    //   0 1 2 3 4 5
    // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    // 3 ⋅ X X X X ⋅
    // 4 ⋅ X X X X ⋅
    // 5 ⋅ X X X X ⋅
    // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    let mut grid_1 = grid_empty.clone();
    grid_1.set(
        &PavingSelector::<(), ()>::empty()
            .dim::<i32>(1..5)
            .dim::<i32>(3..6),
        true,
    );
    assert_ne!(grid_empty, grid_1);

    //   0 1 2 3 4 5
    // 2 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    // 3 ⋅ A # # B ⋅
    // 4 ⋅ C C C C ⋅
    // 5 ⋅ C C C C ⋅
    // 6 ⋅ ⋅ ⋅ ⋅ ⋅ ⋅
    let mut grid_2 = grid_empty.clone();
    grid_2.set(&PavingSelector::empty().dim(1..4).dim(3..4), true); // A & #
    grid_2.set(&PavingSelector::empty().dim(2..5).dim(3..4), true); // B & #
    grid_2.set(&PavingSelector::empty().dim(1..5).dim(4..6), true); // C
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
    grid.set(&PavingSelector::empty().dim(1..4).dim(3..4), true); // A & #
    grid.set(&PavingSelector::empty().dim(2..5).dim(3..4), true); // B & #
    grid.set(&PavingSelector::empty().dim(1..5).dim(4..6), true); // C

    assert_eq!(
        grid.pop_selector().unwrap(),
        PavingSelector::empty().dim(1..5).dim(3..6),
    );

    assert_eq!(grid, grid_empty);
    assert_eq!(grid.pop_selector(), None);
}
