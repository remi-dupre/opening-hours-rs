//! Run stub generation.
//! See https://github.com/Jij-Inc/pyo3-stub-gen

use std::fs::File;
use std::io::Read;

const STUB_SOURCE_PATH: &str = "opening_hours_py.pyi";
const STUB_TARGET_PATH: &str = "../opening_hours.pyi";

// ⚠️  Do not copy this code as it is optimized for concision and not performance.
fn load_file(path: &str) -> Vec<u8> {
    let mut file = File::open(path).unwrap_or_else(|err| panic!("could not load {path:?}: {err}"));
    let mut res = Vec::new();

    file.read_to_end(&mut res)
        .unwrap_or_else(|err| panic!("could not read content from {path:?}: {err}"));

    res
}

fn main() -> pyo3_stub_gen::Result<()> {
    let is_subcommand_check = match std::env::args().nth(1).as_deref() {
        None => false,
        Some("check") => true,
        Some(x) => panic!("Unknown subcommand {x:?}"),
    };

    let stub = opening_hours::stub_info()?;
    stub.generate()?;

    let ruff_output = std::process::Command::new("poetry")
        .args(["run", "ruff", "format", STUB_SOURCE_PATH])
        .output()
        .expect("failed to format stub with ruff");

    println!("Ran ruff: {}", String::from_utf8_lossy(&ruff_output.stdout));
    let changed = load_file(STUB_SOURCE_PATH) != load_file(STUB_TARGET_PATH);

    if changed {
        if is_subcommand_check {
            println!(
                "--- SOURCE:\n{}",
                String::from_utf8_lossy(&load_file(STUB_SOURCE_PATH))
            );

            println!(
                "--- TARGET:\n{}",
                String::from_utf8_lossy(&load_file(STUB_TARGET_PATH))
            );

            std::fs::remove_file(STUB_SOURCE_PATH).expect("could not remove stub file");
            panic!("File changed!");
        } else {
            println!("Installing new stub file to {STUB_TARGET_PATH}.");

            std::fs::rename(STUB_SOURCE_PATH, STUB_TARGET_PATH)
                .expect("could not move stub to project's root");
        }
    } else {
        println!("No change detected.");
        std::fs::remove_file(STUB_SOURCE_PATH).expect("could not remove stub file");
    }

    Ok(())
}
