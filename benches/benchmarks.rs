use opening_hours::parser::parse;

use chrono::NaiveDateTime;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SCH_24_7: &str = "24/7";
const SCH_ADDITION: &str = "10:00-12:00 open, 14:00-16:00 unknown, 16:00-23:00 closed";

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    group.bench_function("24_7", |b| b.iter(|| parse(black_box(SCH_24_7)).unwrap()));

    group.bench_function("addition", |b| {
        b.iter(|| parse(black_box(SCH_ADDITION)).unwrap())
    });
}

fn bench_eval(c: &mut Criterion) {
    let date_time = NaiveDateTime::parse_from_str("2020-06-01 12:03", "%Y-%m-%d %H:%M").unwrap();
    let sch_24_7 = parse(SCH_24_7).unwrap();
    let sch_addition = parse(SCH_ADDITION).unwrap();

    {
        let mut group = c.benchmark_group("is_open");

        group.bench_function("24_7", |b| {
            b.iter(|| black_box(&sch_24_7).is_open(black_box(date_time)))
        });

        group.bench_function("addition", |b| {
            b.iter(|| black_box(&sch_addition).is_open(black_box(date_time)))
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
    }
}

criterion_group!(benches, bench_parse, bench_eval);
criterion_main!(benches);
