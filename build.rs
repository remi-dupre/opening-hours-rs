use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::PathBuf;

use chrono::NaiveDate;
use flate2::write::DeflateEncoder;
use flate2::Compression;

use compact_calendar::CompactCalendar;

const PATH_ENV_IN_OUT: [[&str; 3]; 2] = [
    ["PUBLIC", "data/holidays_public.txt", "holidays_public.bin"],
    ["SCHOOL", "data/holidays_school.txt", "holidays_school.bin"],
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir: PathBuf = env::var_os("OUT_DIR")
        .expect("cargo build didn't specify an `OUT_DIR` variable")
        .into();

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
        println!("cargo::rerun-if-changed={}", out_path.display());

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

    println!("cargo::rerun-if-changed=build.rs");
    Ok(())
}
