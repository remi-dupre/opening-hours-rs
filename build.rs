use std::collections::HashMap;
use std::env;
use std::fs::{create_dir, File};
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::PathBuf;

use chrono::{Datelike, NaiveDate};

const HOLIDAYS_PATH: &str = "data/holidays.txt";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("holidays");
    create_dir(&out_dir).ok();

    // Load dates into an hashmap
    let mut regions: HashMap<String, Vec<i32>> = HashMap::new();
    let lines = BufReader::new(File::open(HOLIDAYS_PATH)?).lines();

    for line in lines {
        let line = line?;
        let mut line = line.splitn(2, ' ');

        let region = line.next().unwrap();
        let date = NaiveDate::parse_from_str(line.next().unwrap(), "%Y-%m-%d")?;

        regions
            .entry(region.to_string())
            .or_default()
            .push(date.num_days_from_ce());
    }

    // Build binary data for all regions
    for (region, dates) in regions {
        let out_path = out_dir.join(format!("{}.bin", region));
        let mut output = BufWriter::new(File::create(out_path)?);

        for date in dates {
            output.write_all(&date.to_le_bytes())?;
        }
    }

    println!("cargo:rustc-env=HOLIDAYS_DIR={}", out_dir.display());
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
