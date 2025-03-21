use crate::{datetime, Context, OpeningHours};
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

#[test]
fn outside_date_bounds() -> Result<(), Error> {
    let before_bounds = datetime!("1789-07-14 12:00");

    let after_bounds = datetime!("9999-01-01 12:00")
        .checked_add_days(chrono::Days::new(366))
        .unwrap();

    assert!(OpeningHours::parse("24/7")?.is_closed(before_bounds));
    assert!(OpeningHours::parse("24/7")?.is_closed(after_bounds));

    assert_eq!(
        OpeningHours::parse("3000")?
            .next_change(before_bounds)
            .unwrap(),
        datetime!("3000-01-01 00:00")
    );

    // TODO
    // assert_eq!(
    //     OpeningHours::parse("24/7")?
    //         .next_change(before_bounds)
    //         .unwrap(),
    //     datetime!("1900-01-01 00:00")
    // );

    assert!(OpeningHours::parse("24/7")?
        .next_change(after_bounds)
        .is_none());

    Ok(())
}

#[test]
// TODO?
// fn with_max_interval_size() {
//     let ctx = Context::default().approx_bound_interval_size(chrono::TimeDelta::days(366));
//
//     let oh = OpeningHours::parse("2024-2030Jun open")
//         .unwrap()
//         .with_context(ctx);
//
//     assert_eq!(
//         oh.next_change(datetime!("2025-05-01 12:00")).unwrap(),
//         datetime!("2025-06-01 00:00"),
//     );
//
//     assert!(oh.next_change(datetime!("2000-05-01 12:00")).is_none());
//     assert!(oh.next_change(datetime!("2030-07-01 12:00")).is_none());
// }
#[test]
fn only_comment_changes_time() {
    let oh: OpeningHours = r#"00:00-14:00 "may open earlier", 14:00-24:00"#.parse().unwrap();

    assert_eq!(
        oh.next_change(datetime!("2024-01-01 12:00")).unwrap(),
        datetime!("2024-01-01 14:00")
    );

    assert_eq!(
        oh.next_change(datetime!("2024-01-01 16:00")).unwrap(),
        datetime!("2024-01-02 00:00")
    );
}

#[test]
fn only_comment_changes_date() {
    let oh: OpeningHours = r#"24/7 "aaa" ; Mar "bbb""#.parse().unwrap();

    assert_eq!(
        oh.next_change(datetime!("2024-01-01 12:00")).unwrap(),
        datetime!("2024-03-01 00:00")
    );

    assert_eq!(
        oh.next_change(datetime!("2024-03-15 12:00")).unwrap(),
        datetime!("2024-04-01 00:00")
    );
}

#[test]
fn single_closed_comment() -> Result<(), Error> {
    let expr = r#"01:00-03:00 closed "aaa""#;

    assert_eq!(
        OpeningHours::parse(expr)?
            .next_change(datetime!("2024-01-01 02:00"))
            .unwrap(),
        datetime!("2024-01-01 03:00"),
    );

    assert_eq!(
        OpeningHours::parse(expr)?
            .next_change(datetime!("2024-01-01 12:00"))
            .unwrap(),
        datetime!("2024-01-02 01:00"),
    );

    Ok(())
}

#[test]
fn open_comment_full_day() -> Result<(), Error> {
    let expr = r#""aaa" ; Mar"#;

    assert_eq!(
        OpeningHours::parse(expr)?
            .next_change(datetime!("2024-01-01 12:00"))
            .unwrap(),
        datetime!("2024-03-01 00:00"),
    );

    assert_eq!(
        OpeningHours::parse(expr)?
            .next_change(datetime!("2024-03-15 12:00"))
            .unwrap(),
        datetime!("2024-04-01 00:00"),
    );

    Ok(())
}
