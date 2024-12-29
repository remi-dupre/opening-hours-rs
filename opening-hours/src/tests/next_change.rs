use crate::{datetime, OpeningHours};
use opening_hours_syntax::error::Error;

#[test]
fn always_open() -> Result<(), Error> {
    assert!("24/7"
        .parse::<OpeningHours>()?
        .next_change(datetime!("2019-02-10 11:00"))
        .is_none());

    Ok(())
}

#[test]
fn date_limit_exceeded() -> Result<(), Error> {
    assert!("24/7"
        .parse::<OpeningHours>()?
        .next_change(datetime!("+10000-01-01 00:00"))
        .is_none());
    Ok(())
}

#[test]
fn skip_year_interval() -> Result<(), Error> {
    assert_eq!(
        "2020,8000-9000 10:00-22:00"
            .parse::<OpeningHours>()?
            .next_change(datetime!("2021-02-09 21:00"))
            .unwrap(),
        datetime!("8000-01-01 10:00")
    );

    assert_eq!(
        "2021,8000-9000 10:00-22:00"
            .parse::<OpeningHours>()?
            .next_change(datetime!("2021-02-09 21:00"))
            .unwrap(),
        datetime!("2021-02-09 22:00")
    );

    assert_eq!(
        "2000-3000"
            .parse::<OpeningHours>()?
            .next_change(datetime!("2021-02-09 21:00"))
            .unwrap(),
        datetime!("3001-01-01 00:00")
    );

    assert_eq!(
        "2000-3000/42"
            .parse::<OpeningHours>()?
            .next_change(datetime!("2021-02-09 21:00"))
            .unwrap(),
        datetime!("2042-01-01 00:00")
    );

    assert_eq!(
        "2000-3000/21"
            .parse::<OpeningHours>()?
            .next_change(datetime!("2021-02-09 21:00"))
            .unwrap(),
        datetime!("2022-01-01 00:00")
    );

    Ok(())
}
