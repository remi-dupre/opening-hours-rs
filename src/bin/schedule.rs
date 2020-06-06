use std::env;

use chrono::{Duration, Local};

use opening_hours::parse;

fn main() {
    let expression = env::args().nth(1).expect("Usage: ./schedule <EXPRESSION>");
    let start_date = Local::now().naive_local().date();

    let time_domain = match parse(&expression) {
        Ok(val) => val,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    for day in 0..7 {
        let date = start_date + Duration::days(day);
        let schedule = time_domain.schedule_at(date);

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
