use crate::error::Result;
use crate::parser::parse;

macro_rules! ex {
    ( $( $tt: expr ),* $( , )? ) => {
        (file!(), line!() $( , $tt )*)
    };
}

const EXAMPLES: &[(&str, u32, &str, &str)] = &[
    ex!("Sa; 24/7", "24/7"),
    ex!("06:00+;24/7", "06:00+ ; 24/7"),
    ex!("06:00-24:00;24/7", "24/7"),
    ex!("Tu-Mo", "24/7"),
    ex!("2022;Fr", "2022 ; 1900-2021,2023-9999 Fr"),
    ex!("Mo,Th open ; Tu,Fr-Su open", "Mo-Tu,Th-Su"),
    ex!("Mo-Fr 10:00-14:00 ; We-Su 10:00-14:00", "10:00-14:00"),
    ex!("Mo,Tu,We,Th,Fr,Sa,Su 10:00-21:00", "10:00-21:00"),
    ex!("5554Mo;5555", "5554-5555 Mo ; 5555 Tu-Su"),
    ex!("4444-4405", "1900-4405,4444-9999"),
    ex!("Jun24:00+", "Jun 24:00+"),
    ex!("week02-02/7", "week02-02/7"),
    ex!(
        "Mo 10:00-21:00; Tu,We,Th,Fr,Sa,Su 10:00-21:00",
        "10:00-21:00"
    ),
    ex!(
        "Nov-Mar Mo-Fr 10:00-16:00 ; Apr-Nov Mo-Fr 08:00-18:00",
        "Apr-Nov Mo-Fr 08:00-18:00 ; Jan-Mar,Dec Mo-Fr 10:00-16:00"
    ),
    ex!(
        "Apr-Oct Mo-Fr 08:00-18:00 ; Mo-Fr 10:00-16:00 open",
        "Mo-Fr 10:00-16:00"
    ),
    ex!(
        "Mo-Fr 10:00-16:00 open ; Apr-Oct Mo-Fr 08:00-18:00",
        "Apr-Oct Mo-Fr 08:00-18:00 ; Jan-Mar,Nov-Dec Mo-Fr 10:00-16:00"
    ),
    ex!(
        "Mo-Su 00:00-01:00, 10:30-24:00 ; PH off ; 2021 Apr 10 00:00-01:00 ; 2021 Apr 11-16 off ; 2021 Apr 17 10:30-24:00",
        "00:00-01:00, 10:30-24:00 ; PH closed ; 2021 Apr 10 00:00-01:00 ; 2021 Apr 11-2021 Apr 16 closed ; 2021 Apr 17 10:30-24:00"
    ),
    ex!(
        "week2Mo;Jun;Fr",
        "Jun ; Jan-May,Jul-Dec week02 Mo,Fr ; Jan-May,Jul-Dec week01,03-53 Fr"
    ),
];

#[test]
fn simplify_already_minimal() -> Result<()> {
    for (file, line, _, example) in EXAMPLES {
        assert_eq!(
            parse(example)?.simplify().to_string(),
            *example,
            "error with example from {file}:{line}",
        );
    }

    Ok(())
}

#[test]
fn simplify() -> Result<()> {
    for (file, line, expr, simplified) in EXAMPLES {
        assert_eq!(
            parse(expr)?.simplify().to_string(),
            *simplified,
            "error with example from {file}:{line}",
        );
    }

    Ok(())
}
