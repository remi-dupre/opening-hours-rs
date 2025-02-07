use crate::error::Result;
use crate::parser::parse;

const EXAMPLES_ALREADY_SIMPLIFIED: &[&str] = &["24/7 open"];

const EXAMPLES: &[[&str; 2]] = &[
    ["Mo,Th open ; Tu,Fr-Su open", "Mo-Tu,Th-Su open"],
    ["Mo-Fr 10:00-14:00 ; We-Su 10:00-14:00", "10:00-14:00 open"],
    ["Mo,Tu,We,Th,Fr,Sa,Su 10:00-21:00", "10:00-21:00 open"],
    [
        "Mo 10:00-21:00; Tu,We,Th,Fr,Sa,Su 10:00-21:00",
        "10:00-21:00 open",
    ],
    [
        "Nov-Mar Mo-Fr 10:00-16:00 ; Apr-Nov Mo-Fr 08:00-18:00",
        "Apr-Oct Mo-Fr 08:00-18:00 open; Jan-Mar,Oct-Apr Mo-Fr 10:00-18:00 open",
    ],
    [
        "Apr-Oct Mo-Fr 08:00-18:00 ; Mo-Fr 10:00-16:00 open",
        "Apr-Oct Mo-Fr 08:00-18:00 open; Jan-Mar,Oct-Apr Mo-Fr 11:00-18:00 open",
    ],
    // TOOD: time should not be part of dimensions ; it should be part of the
    // inside value (we filter on date and THEN compute opening hours)
];

#[test]
fn simplify_already_minimal() -> Result<()> {
    for example in EXAMPLES_ALREADY_SIMPLIFIED {
        assert_eq!(parse(example)?.simplify().to_string(), *example);
    }

    Ok(())
}

#[test]
fn merge_weekdays() -> Result<()> {
    for [expr, simplified] in EXAMPLES {
        assert_eq!(parse(expr)?.simplify().to_string(), *simplified);
    }

    Ok(())
}
