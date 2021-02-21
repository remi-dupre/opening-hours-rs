use std::env;

use chrono::{Duration, Local};

use opening_hours::parse;

fn main() {
    let expression = env::args().nth(1).expect("Usage: ./schedule <EXPRESSION>");
    let start_datetime = Local::now().naive_local();
    let start_date = start_datetime.date();

    let oh = match parse(&expression) {
        Ok(val) => val.with_region("FR"),
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    println!(" - expression: {:?}", expression);
    println!(" - date: {:?}", start_date);
    println!(" - current status: {:?}", oh.state(start_datetime).unwrap());
    println!(
        " - next change: {:?}",
        oh.next_change(start_datetime).unwrap()
    );

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
