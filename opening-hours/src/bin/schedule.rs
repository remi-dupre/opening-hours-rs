use std::env;

use chrono::{Duration, Local};

use opening_hours::localization::Country;
use opening_hours::{Context, OpeningHours};

const COUNTRY: Country = Country::FR;

fn main() {
    let expression = env::args().nth(1).expect("Usage: ./schedule <EXPRESSION>");
    let start_datetime = Local::now().naive_local();
    let start_date = start_datetime.date();

    let oh = match expression.parse::<OpeningHours>() {
        Ok(val) => val.with_context(Context::default().with_holidays(COUNTRY.holidays())),
        Err(err) => {
            panic!("{err}");
        }
    };

    println!(" - expression: {oh}");
    println!(" - normalized: {}", oh.normalize());
    println!(" - date: {start_date:?}");
    let (kind, comment) = oh.state(start_datetime);
    println!(" - current status: {kind:?}");

    if !comment.is_empty() {
        println!(" - current comment: {comment}");
    }

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

            if !tr.comment.is_empty() {
                print!(" ({})", tr.comment);
            }

            println!()
        }
    }
}
