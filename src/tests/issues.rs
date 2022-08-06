//! Test from Github issues.
//! See https://github.com/remi-dupre/opening-hours-rs/issues

use opening_hours_syntax::error::Error;

use crate::{datetime, OpeningHours};

#[test]
fn gh023_handling_of_spaces() -> Result<(), Error> {
    let oh = OpeningHours::parse("Apr 1 - Nov 3 00:00-24:00")?;
    let start = datetime!("2018-06-11 00:00");
    let expected_end = datetime!("2018-11-04 00:00");
    assert_eq!(oh.next_change(start).unwrap(), expected_end);
    Ok(())
}
