extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate chrono;

pub mod extended_time;
pub mod parser;
pub mod schedule;
pub mod time_domain;
pub mod time_selector;
pub mod utils;

use chrono::{Duration, NaiveDate};

fn main() {
    let res = parser::parse(
        r#"2020-2050week24-50,30-40:Mo1-3,5 08:00-13:00,14:00-17:00,13:00-14:00 unknown "not on bad weather days!""#,
    )
    .unwrap();

    println!("{:#?}", &res);

    let mut date = NaiveDate::from_ymd(2020, 6, 1);

    for _ in 0..31 {
        println!(
            "{:?}: {:?}",
            date,
            res.schedule_at(date).into_iter().collect::<Vec<_>>()
        );
        date += Duration::days(1);
    }

    for range in res.iter_from(date).take(1000) {
        println!("{:?}", range);
    }
}
