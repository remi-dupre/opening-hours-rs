use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use rustc_version::{version_meta, Channel};

use compact_calendar::CompactCalendar;

fn detect_build_channel() {
    let channel = match version_meta().unwrap().channel {
        Channel::Stable => "CHANNEL_STABLE",
        Channel::Beta => "CHANNEL_BETA",
        Channel::Nightly => "CHANNEL_NIGHTLY",
        Channel::Dev => "CHANNEL_DEV",
    };

    println!("cargo:rustc-cfg={}", channel)
}

fn generate_holiday_database(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    const PATH_ENV_IN_OUT: [[&str; 3]; 2] = [
        [
            "PUBLIC",
            "opening-hours/data/holidays_public.txt",
            "holidays_public.bin",
        ],
        [
            "SCHOOL",
            "opening-hours/data/holidays_school.txt",
            "holidays_school.bin",
        ],
    ];

    for [env, in_path, out_path] in &PATH_ENV_IN_OUT {
        // Load dates into an map
        let mut region_dates: BTreeMap<String, Vec<NaiveDate>> = BTreeMap::new();
        let lines = BufReader::new(File::open(in_path)?).lines();

        for line in lines {
            let line = line?;
            let mut line = line.splitn(2, ' ');
            let region = line.next().expect("missing region");
            let date = NaiveDate::parse_from_str(line.next().expect("missing date"), "%Y-%m-%d")?;

            region_dates
                .entry(region.to_string())
                .or_default()
                .push(date);
        }

        // Build binary data for all regions
        let out_path = out_dir.join(out_path);

        let mut output = DeflateEncoder::new(
            BufWriter::new(File::create(&out_path)?),
            Compression::best(),
        );

        let regions_order: Vec<_> = region_dates
            .into_iter()
            .map(|(region, dates)| {
                let mut calendar = CompactCalendar::default();

                for date in dates {
                    calendar.insert(date);
                }

                calendar.serialize(&mut output)?;
                Ok::<_, Box<dyn std::error::Error>>(region)
            })
            .collect::<Result<_, _>>()?;

        output.finish()?;
        println!("cargo::rerun-if-changed={in_path}");

        // Export path values
        println!(
            "cargo::rustc-env=HOLIDAYS_{env}_FILE={}",
            out_path.display()
        );

        println!(
            "cargo::rustc-env=HOLIDAYS_{env}_REGIONS={}",
            regions_order.join(",")
        );
    }

    Ok(())
}

#[cfg(feature = "auto-country")]
fn generate_coutry_bounds_database(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    let out_path = out_dir.join("country_boundaries.bin");
    let mut output = DeflateEncoder::new(File::create(&out_path)?, Compression::best());
    output.write_all(country_boundaries::BOUNDARIES_ODBL_60X30)?;
    output.finish()?;

    println!(
        "cargo::rustc-env=COUNTRY_BOUNDS_FILE={}",
        out_path.display()
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir: PathBuf = env::var_os("OUT_DIR")
        .expect("cargo build didn't specify an `OUT_DIR` variable")
        .into();

    #[cfg(feature = "auto-country")]
    generate_coutry_bounds_database(&out_dir)?;

    generate_holiday_database(&out_dir)?;
    detect_build_channel();
    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
