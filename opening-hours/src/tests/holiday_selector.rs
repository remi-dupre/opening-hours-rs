use opening_hours_syntax::error::Result;
use opening_hours_syntax::rules::RuleKind::*;

use crate::localization::Country;
use crate::schedule_at;

#[test]
fn public_holidays() -> Result<()> {
    let expr = "2020:10:00-12:00; PH off";

    // The 14th of July is a holiday in France

    assert_eq!(
        schedule_at!(expr, "2020-07-14", region = Country::FR),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(expr, "2020-07-14", region = Country::US),
        schedule! { 10,00 => Open => 12,00 }
    );

    // Independence Day is a federal holiday. If July 4 is a Saturday, it is
    // observed on Friday, July 3.

    assert_eq!(
        schedule_at!(expr, "2020-07-03", region = Country::US),
        schedule! {}
    );

    assert_eq!(
        schedule_at!(expr, "2020-07-04", region = Country::US),
        schedule! { 10,00 => Open => 12,00 }
    );

    Ok(())
}

#[test]
fn regional_holidays() -> Result<()> {
    // International Women's Day is only a holiday in Berlin for Germany
    assert_eq!(
        schedule_at!("10:00-12:00; PH off", "2025-03-08", region = Country::DE),
        schedule! { 00,00 => Unknown => 24,00 }
    );

    assert_eq!(
        schedule_at!(
            "08:00-18:00, PH 12:00-14:00 off",
            "2025-03-08",
            region = Country::DE
        ),
        schedule! { 8,00 => Open => 12,00 => Unknown => 14,00 => Open => 18,00 }
    );

    Ok(())
}
