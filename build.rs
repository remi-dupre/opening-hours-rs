use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::PathBuf;

use chrono::{Datelike, NaiveDate};
use flate2::write::ZlibEncoder;
use flate2::Compression;

use compact_calendar::CompactCalendar;

/// Input path to read holidays from
const HOLIDAYS_PATH: &str = "data/holidays.txt";

/// Output path for holidays
const OUTPUT_FILE: &str = "holidays.bin";

/// Watched path for cargo to rebuild holidays
const WATCH_PATHS: &[&str] = &["build.rs", "data/holidays.txt"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir: PathBuf = env::var_os("OUT_DIR")
        .expect("cargo build didn't specify an `OUT_DIR` variable")
        .into();

    // Load dates into an hashmap
    let mut region_dates: HashMap<String, Vec<NaiveDate>> = HashMap::new();
    let lines = BufReader::new(File::open(HOLIDAYS_PATH)?).lines();

    for line in lines {
        let line = line?;
        let mut line = line.splitn(2, ' ');

        let region = line.next().unwrap();
        let date = NaiveDate::parse_from_str(line.next().unwrap(), "%Y-%m-%d")?;

        region_dates
            .entry(region.to_string())
            .or_default()
            .push(date);
    }

    // Build binary data for all regions
    let out_path = out_dir.join(OUTPUT_FILE);

    let mut output = ZlibEncoder::new(
        BufWriter::new(File::create(&out_path)?),
        Compression::best(),
    );

    let regions_order: Vec<_> = region_dates
        .into_iter()
        .map(|(region, dates)| {
            let min_year = dates.iter().map(Datelike::year).min().unwrap_or(2000);
            let max_year = dates.iter().map(Datelike::year).max().unwrap_or(2000);
            let mut calendar = CompactCalendar::new(min_year, max_year);

            for date in dates {
                assert!(calendar.insert(date));
            }

            calendar.serialize(&mut output)?;
            Ok::<_, Box<dyn std::error::Error>>(region)
        })
        .collect::<Result<_, _>>()?;

    output.finish()?;

    // Export path values
    println!("cargo:rustc-env=HOLIDAYS_FILE={}", out_path.display());

    println!(
        "cargo:rustc-env=HOLIDAYS_REGIONS={}",
        regions_order.join(",")
    );

    for path in WATCH_PATHS {
        println!("cargo:rerun-if-changed={path}");
    }

    Ok(())
}
