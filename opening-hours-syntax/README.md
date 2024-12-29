# Rust parser for OSM Opening Hours

[![](https://img.shields.io/crates/v/opening-hours-syntax)][opening-hours-syntax]
[![](https://img.shields.io/docsrs/opening-hours-syntax)][docs]

Parsing component of [opening-hours] crate.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
opening-hours-syntax = "0"
```

And then a basic usage would look like that:

```rust
use opening_hours_syntax::parse;

// Opens until 18pm during the week and until 12am the week-end.
const OH: &str = "Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00";

fn main() {
    let oh = parse(&OH).unwrap();
    eprintln!("{:?}", oh);
}
```

## Supported features

### Logging

The **log** feature can be enabled to emit warnings the [crate-log] crate.

[docs]: https://docs.rs/opening-hours-syntax "Documentation"
[opening-hours]: https://crates.io/crates/opening-hours "Root Package"
[opening-hours-syntax]: https://crates.io/crates/opening-hours "Package"
