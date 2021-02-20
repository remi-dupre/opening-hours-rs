Rust implementation for OSM Opening Hours
=========================================

[![Crates.io](https://img.shields.io/crates/v/opening-hours)](https://crates.io/crates/opening-hours)
[![PyPI](https://img.shields.io/pypi/v/opening-hours-py)](https://pypi.org/project/opening-hours-py/)
[![docs.rs](https://img.shields.io/docsrs/opening-hours)](https://docs.rs/opening-hours/)
[![Crates.io](https://img.shields.io/crates/l/opening-hours)](https://crates.io/crates/opening-hours)
[![Codecov](https://img.shields.io/codecov/c/github/remi-dupre/opening-hours-rs)](https://app.codecov.io/gh/remi-dupre/opening-hours-rs)


**Python bindings can be found [here](https://github.com/remi-dupre/opening-hours-rs/tree/master/python)**

A Rust library for parsing and working with OSM's opening hours field. You can
find its specification [here](https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification)
and the reference JS library [here](https://github.com/opening-hours/opening_hours.js).

Note that the specification is quite messy and that the JS library takes
liberty to extend it quite a lot. This means that most of the real world data
don't actually comply to the very restrictive grammar detailed in the official
specification. This library tries to fit with the real world data while
remaining as close as possible to the core specification.


Usage
-----

Add this to your `Cargo.toml`:

```toml
[dependancies]
opening-hours = "0"
```

Here's a simple example that parse an opening hours description and displays
its current status and date for next change:

```rust
use chrono::Local;
use opening_hours::parse;

// Opens until 18pm during the week and until 12am the week-end.
const OH: &str = "Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00";

fn main() {
    let oh = parse(&expression).unwrap();
    let date = Local::now().naive_local();
    println!("Current status is {:?}", oh.state(date).unwrap());
    println!("This will change at {:?}", oh.next_change(date).unwrap());
}
```
