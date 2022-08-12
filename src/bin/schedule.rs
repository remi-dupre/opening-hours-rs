use std::env;

use chrono::{Duration, Local};

use opening_hours::OpeningHours;

const REGION: &str = "FR";

fn main() {
    let expression = env::args().nth(1).expect("Usage: ./schedule <EXPRESSION>");
    let start_datetime = Local::now().naive_local();
    let start_date = start_datetime.date();

    let oh = match OpeningHours::parse(&expression) {
        Ok(val) => val.with_region(REGION),
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let next_change = oh.next_change(start_datetime).unwrap();

    println!(" - expression: {expression:?}");
    println!(" - date: {start_date:?}");
    println!(" - loaded holidays for {REGION}: {}", oh.holidays().count());
    println!(" - current status: {:?}", oh.state(start_datetime).unwrap());
    println!(" - next change: {next_change:?}");

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
