use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::schedule_at;

#[test]
fn holidays() -> Result<(), Error> {
    // The 14th of July is a holiday in France
    assert_eq!(
        schedule_at!("2020:10:00-12:00; PH off", "2020-07-14", "FR"),
        schedule! { 00,00 => Closed => 24,00 }
    );

    assert_eq!(
        schedule_at!("2020:10:00-12:00; PH off", "2020-07-14", "US"),
        schedule! { 10,00 => Open => 12,00 }
    );

    // The 14th of July is a holiday in the US
    assert_eq!(
        schedule_at!("2020:10:00-12:00; PH off", "2020-07-04", "US"),
        schedule! { 00,00 => Closed => 24,00 }
    );

    assert_eq!(
        schedule_at!("2020:10:00-12:00; PH off", "2020-07-04", "FR"),
        schedule! { 10,00 => Open => 12,00 }
    );

    Ok(())
}
