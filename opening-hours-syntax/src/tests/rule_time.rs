use rstest::rstest;

use crate::rules::time::TimeEvent::*;
use crate::rules::time::VariableTime;

#[rstest]
#[case(VariableTime { event: Dusk, offset: 0 }, VariableTime{ event: Dusk, offset: 30 })]
#[case(VariableTime { event: Dusk, offset: -30 }, VariableTime{ event: Dusk, offset: 30 })]
#[case(VariableTime { event: Dusk, offset: -30 }, VariableTime{ event: Dusk, offset: 0 })]
#[case(VariableTime { event: Dawn, offset: 0 }, VariableTime{ event: Dusk, offset: 0 })]
#[case(VariableTime { event: Dawn, offset: -30 }, VariableTime{ event: Dusk, offset: 30 })]
fn variable_time_order(#[case] x: VariableTime, #[case] y: VariableTime) {
    assert!(
        x.is_before(&y),
        "variable times should be ordered: {x:?} < {y:?}"
    )
}

#[rstest]
#[case(VariableTime { event: Dawn, offset: 30 }, VariableTime{ event: Dusk, offset: -30 })]
#[case(VariableTime { event: Dawn, offset: 30 }, VariableTime{ event: Dusk, offset: -30 })]
fn variable_time_order_partial(#[case] x: VariableTime, #[case] y: VariableTime) {
    assert!(
        !x.is_before(&y) && !y.is_before(&x),
        "variable times should not be comparable: {x:?} and {y:?}",
    )
}
