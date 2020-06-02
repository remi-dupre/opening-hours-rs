extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate chrono;

pub mod parser;
pub mod time_domain;
pub mod time_selector;

use chrono::{Duration, NaiveDate};

fn main() {
    let res = parser::parse(
        r#"2020-2050week24-50,30-40:Mo1-3,5 08:00-13:00,14:00-17:00 unknown "not on bad weather days!""#,
    )
    .unwrap();

    println!("{:#?}", &res);

    let mut date = NaiveDate::from_ymd(2020, 6, 1);

    for _ in 0..31 {
        println!("{:?}: {}", date, res.feasible_date(date));
        date += Duration::days(1);
    }
}
