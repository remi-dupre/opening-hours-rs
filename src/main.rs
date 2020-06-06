extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate chrono;

pub mod day_selector;
pub mod extended_time;
pub mod parser;
#[macro_use]
pub mod schedule;
pub mod time_domain;
pub mod time_selector;
pub mod utils;

#[cfg(test)]
mod tests;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

fn main() {
    let res = parser::parse(r#"Mo-Su 11:00-19:00 "salut", Mo-Fr 18:00-25:00 "lol""#)
        .map_err(|err| {
            println!("Got Parsing Error:");
            println!("{}", err);
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

    for range in res.iter_from(NaiveDateTime::new(date, time)) {
        println!("{:?}", range);
    }
}
