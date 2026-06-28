use std::env;

use cbindgen::{Config, Language, RenameRule};

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut config = Config::default();
    config.enumeration.rename_variants = RenameRule::QualifiedScreamingSnakeCase;

    cbindgen::Builder::new()
        .with_config(config)
        .with_language(Language::C)
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("bindings.h");
}
