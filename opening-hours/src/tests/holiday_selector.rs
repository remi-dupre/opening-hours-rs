use opening_hours_syntax::error::Error;
use opening_hours_syntax::rules::RuleKind::*;

use crate::localization::Country;
use crate::schedule_at;

#[test]
fn holidays() -> Result<(), Error> {
    // The 14th of July is a holiday in France
    assert_eq!(
        schedule_at!(
            "2020:10:00-12:00; PH off",
            "2020-07-14",
            region = Country::FR
        ),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(
            "2020:10:00-12:00; PH off",
            "2020-07-14",
            region = Country::US
        ),
        schedule! { 10,00 => Open => 12,00 }
    );

    // Independence Day is a federal holiday. If July 4 is a Saturday, it is
    // observed on Friday, July 3.
    assert_eq!(
        schedule_at!(
            "2020:10:00-12:00; PH off",
            "2020-07-03",
            region = Country::US
        ),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(
            "2020:10:00-12:00; PH off",
            "2020-07-04",
            region = Country::US
        ),
        schedule! { 10,00 => Open => 12,00 }
    );

    Ok(())
}
