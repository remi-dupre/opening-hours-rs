use opening_hours::localization::{Coordinates, Country};
use opening_hours::{Context, OpeningHours};

use chrono::NaiveDateTime;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SCH_24_7: &str = "24/7";
const SCH_ADDITION: &str = "10:00-12:00 open, 14:00-16:00 unknown, 16:00-23:00 closed";
const SCH_HOLIDAY: &str = "PH";
const SCH_JAN_DEC: &str = "Jan-Dec";
const PARIS_COORDS: Coordinates = Coordinates::new(48.8535, 2.34839).unwrap();

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    group.bench_function("24_7", |b| {
        b.iter(|| OpeningHours::parse(black_box(SCH_24_7)).unwrap())
    });

    group.bench_function("addition", |b| {
        b.iter(|| OpeningHours::parse(black_box(SCH_ADDITION)).unwrap())
    });
}

fn bench_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("context");

    group.bench_function("infer_from_coords", |b| {
        b.iter(|| Context::from_coords(black_box(PARIS_COORDS)))
    });
}

fn bench_eval(c: &mut Criterion) {
    let fr_context = Context::default().with_holidays(Country::FR.holidays());
    let date_time = NaiveDateTime::parse_from_str("2021-02-01 12:03", "%Y-%m-%d %H:%M").unwrap();

    let expressions = [
        ("24_7", OpeningHours::parse(SCH_24_7).unwrap()),
        ("addition", OpeningHours::parse(SCH_ADDITION).unwrap()),
        ("holidays", OpeningHours::parse(SCH_HOLIDAY).unwrap()),
        (
            "jan-dec",
            OpeningHours::parse(SCH_JAN_DEC)
                .unwrap()
                .with_context(fr_context),
        ),
    ];

    {
        let mut group = c.benchmark_group("is_open");

        for (slug, expr) in &expressions {
            group.bench_function(*slug, |b| {
                b.iter(|| black_box(&expr).is_open(black_box(date_time)))
            });
        }
    }

    {
        let mut group = c.benchmark_group("next_change");

        for (slug, expr) in &expressions {
            group.bench_function(*slug, |b| {
                b.iter(|| black_box(black_box(&expr).next_change(black_box(date_time))))
            });
        }
    }

    {
        let mut group = c.benchmark_group("normalize");

        for (slug, expr) in &expressions {
            group.bench_function(*slug, |b| {
                b.iter(|| black_box(black_box(&expr).normalize()))
            });
        }
    }
}

criterion_group!(benches, bench_parse, bench_context, bench_eval);
criterion_main!(benches);
