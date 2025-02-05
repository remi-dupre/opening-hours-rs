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
