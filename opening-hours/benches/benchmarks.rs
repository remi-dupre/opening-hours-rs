use opening_hours::country::Country;
use opening_hours::{Context, OpeningHours};

use chrono::NaiveDateTime;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SCH_24_7: &str = "24/7";
const SCH_ADDITION: &str = "10:00-12:00 open, 14:00-16:00 unknown, 16:00-23:00 closed";
const SCH_HOLIDAY: &str = "PH";
const SCH_JAN_DEC: &str = "Jan-Dec";

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    group.bench_function("24_7", |b| {
        b.iter(|| OpeningHours::parse(black_box(SCH_24_7)).unwrap())
    });

    group.bench_function("addition", |b| {
        b.iter(|| OpeningHours::parse(black_box(SCH_ADDITION)).unwrap())
    });
}

fn bench_eval(c: &mut Criterion) {
    let fr_context = Context::default().with_holidays(Country::FR.holidays());
    let date_time = NaiveDateTime::parse_from_str("2021-02-01 12:03", "%Y-%m-%d %H:%M").unwrap();

    let sch_24_7 = OpeningHours::parse(SCH_24_7).unwrap();
    let sch_addition = OpeningHours::parse(SCH_ADDITION).unwrap();
    let sch_jan_dec = OpeningHours::parse(SCH_JAN_DEC).unwrap();

    let sch_holiday = OpeningHours::parse(SCH_HOLIDAY)
        .unwrap()
        .with_context(fr_context);

    {
        let mut group = c.benchmark_group("is_open");

        group.bench_function("24_7", |b| {
            b.iter(|| black_box(&sch_24_7).is_open(black_box(date_time)))
        });

        group.bench_function("addition", |b| {
            b.iter(|| black_box(&sch_addition).is_open(black_box(date_time)))
        });

        group.bench_function("holiday", |b| {
            b.iter(|| black_box(&sch_holiday).is_open(black_box(date_time)))
        });
    }

    {
        let mut group = c.benchmark_group("next_change");

        group.bench_function("24_7", |b| {
            b.iter(|| black_box(&sch_24_7).next_change(black_box(date_time)))
        });

        group.bench_function("addition", |b| {
            b.iter(|| black_box(&sch_addition).next_change(black_box(date_time)))
        });

        group.bench_function("holiday", |b| {
            b.iter(|| black_box(&sch_holiday).next_change(black_box(date_time)))
        });

        group.bench_function("jan-dec", |b| {
            b.iter(|| black_box(&sch_jan_dec).next_change(black_box(date_time)))
        });
    }
}

criterion_group!(benches, bench_parse, bench_eval);
criterion_main!(benches);
