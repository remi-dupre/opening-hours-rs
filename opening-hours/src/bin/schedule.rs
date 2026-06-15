#![allow(clippy::unwrap_used)]

use std::env;

use chrono::{Duration, Local, NaiveDateTime};

use opening_hours::localization::Country;
use opening_hours::{Context, OpeningHours};
use opening_hours_syntax::Parser;

const COUNTRY: Country = Country::FR;

fn main() {
    let mut args = env::args().skip(1);
    let expression = args.next().expect("Usage: ./schedule <EXPRESSION>");

    let start_datetime = args
        .next()
        .map(|dt_str| NaiveDateTime::parse_from_str(&dt_str, "%Y-%m-%d %H:%M").unwrap())
        .unwrap_or_else(|| Local::now().naive_local());

    let start_date = start_datetime.date();
    let mut warns = 0;

    let mut parser = Parser::default().with_warning_handler(|warning| {
        warns += 1;
        eprintln!(" ! warning: {warning}")
    } as _);

    let oh = match OpeningHours::parse_with(&mut parser, &expression) {
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
            print!(" - {}-{} - {:?}", tr.range.start, tr.range.end, tr.kind);

            if !tr.comment.is_empty() {
                print!(" ({})", tr.comment);
            }

            println!()
        }
    }
}
