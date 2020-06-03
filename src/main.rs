extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate chrono;

pub mod day_selector;
pub mod extended_time;
pub mod parser;
pub mod schedule;
pub mod time_domain;
pub mod time_selector;
pub mod utils;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

fn main() {
    let res =
        parser::parse(r#"Mo 12:00-14:00 open "female only", Mo 14:00-16:00 unknown "male only""#)
            .map_err(|err| {
                println!("Got Parsing Error:");
                println!("{}", err.description);
            })
            .unwrap();

    println!("{:#?}", &res);

    let mut date = NaiveDate::from_ymd(2020, 6, 1);
    let time = NaiveTime::from_hms(23, 59, 59);

    for _ in 0..31 {
        println!(
            "{:?}: {:?}",
            date,
            res.schedule_at(date).into_iter().collect::<Vec<_>>()
        );
        date += Duration::days(1);
    }

    for range in res.iter_from(NaiveDateTime::new(date, time)).take(1000) {
        println!("{:?}", range);
    }
}
