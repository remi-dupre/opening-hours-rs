use std::env;

use chrono::{Duration, Local};

use opening_hours::context::Context;
use opening_hours::country::Country;
use opening_hours::OpeningHours;

const COUNTRY: Country = Country::FR;

fn main() {
    let expression = env::args().nth(1).expect("Usage: ./schedule <EXPRESSION>");
    let start_datetime = Local::now().naive_local();
    let start_date = start_datetime.date();
    println!(" - expression: {expression}");

    let oh = match expression.parse::<OpeningHours>() {
        Ok(val) => val.with_context(Context::default().with_holidays(COUNTRY.holidays())),
        Err(err) => {
            panic!("{err}");
        }
    };

    println!(" - formatted: {oh}");
    println!(" - date: {start_date:?}");
    println!(" - current status: {:?}", oh.state(start_datetime));

    if let Some(next_change) = oh.next_change(start_datetime) {
        println!(" - next change: {next_change:?}");
    }

    for day in 0..7 {
        let date = start_date + Duration::days(day);
        let schedule = oh.schedule_at(date);

        println!("---");
        println!("{}:", date.format("%A, %-d %B, %C%y"));

        if schedule.is_empty() {
            println!(" (empty)");
        }

        for tr in schedule {
            print!(" - {:?} - {:?}", tr.range, tr.kind);

            if !tr.comments.is_empty() {
                print!(" ({})", tr.comments.join(", "));
            }

            println!()
        }
    }
}
