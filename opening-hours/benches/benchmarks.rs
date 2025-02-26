use opening_hours::localization::{Coordinates, Country};
use opening_hours::{Context, OpeningHours};

use chrono::NaiveDateTime;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SAMPLES: &[[&str; 2]] = &[
    ["24_7", "24/7"],
    ["holidays", "Mo-Fr 10:00-18:00 ; PH off"],
    [
        "rule_normal",
        "Mo-Fr 10:00-12:00,14:00-18:00 ; Sa-Su 10:00-14:00 unknown ; Dec31 off",
    ],
    [
        "rule_addition",
        "Mo-Fr 10:00-18:00 , Sa-Su 10:00-14:00 unknown, 12:00-14:00 closed",
    ],
    [
        "rule_fallback",
        "Mo-Fr 10:00-12:00,14:00-18:00 ; Sa 10:00-13:00 || 10:00-12:00 unknown",
    ],
    [
        "huge",
        "Aug:Sa;week50unknown;Nov;2492week9:;:Mo;Fr;1912week48;:Mo;7591-1918week1:;:Mo;week8Sa;1918week3:;:Mo;7191-1911Mo;MayWe;Fr;week2;Feb;Oct;3683;Fr;1915week48;:Mo;5182-1919week1:;:Mo;week8Sa;1918week4:;:Mo;7191-1911;Mo;MayWe;Fr;week2;Feb;Oct;3836week3:;:Th;Su;3818closed; Fr;1917week17",
    ],
];

const PARIS_COORDS: Coordinates = Coordinates::new(48.8535, 2.34839).unwrap();

fn bench_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("context");

    group.bench_function("infer_from_coords", |b| {
        b.iter(|| Context::from_coords(black_box(PARIS_COORDS)))
    });
}

fn bench_sample(c: &mut Criterion) {
    let fr_context = Context::default().with_holidays(Country::FR.holidays());
    let date_time = NaiveDateTime::parse_from_str("2021-02-01 12:03", "%Y-%m-%d %H:%M").unwrap();

    let sample_oh: Vec<_> = SAMPLES
        .iter()
        .map(|[slug, expr]| {
            (
                *slug,
                OpeningHours::parse(expr)
                    .unwrap()
                    .with_context(fr_context.clone()),
            )
        })
        .collect();

    {
        let mut group = c.benchmark_group("parse");

        for [slug, expr] in SAMPLES {
            group.bench_function(*slug, |b| {
                b.iter(|| OpeningHours::parse(black_box(expr)).unwrap())
            });
        }
    }

    {
        let mut group = c.benchmark_group("is_open");

        for (slug, oh) in &sample_oh {
            group.bench_function(*slug, |b| {
                b.iter(|| black_box(&oh).is_open(black_box(date_time)))
            });
        }
    }

    {
        let mut group = c.benchmark_group("next_change");

        for (slug, oh) in &sample_oh {
            group.bench_function(*slug, |b| {
                b.iter(|| black_box(black_box(&oh).next_change(black_box(date_time))))
            });
        }
    }

    {
        let mut group = c.benchmark_group("normalize");

        for (slug, oh) in &sample_oh {
            group.bench_function(*slug, |b| b.iter(|| black_box(black_box(&oh).normalize())));
        }
    }
}

criterion_group!(benches, bench_context, bench_sample);
criterion_main!(benches);
