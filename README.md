Rust implementation for OSM Opening Hours
=========================================

[![](https://img.shields.io/crates/v/opening-hours)][opening-hours]
[![](https://img.shields.io/pypi/v/opening-hours-py)][pypy]
[![](https://img.shields.io/docsrs/opening-hours)][docs]
[![](https://img.shields.io/crates/l/opening-hours)][opening-hours]
[![](https://img.shields.io/codecov/c/github/remi-dupre/opening-hours-rs)][codecov]


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
[dependencies]
opening-hours = "0"
```

Here's a simple example that parse an opening hours description and displays
its current status and date for next change:

```rust
use chrono::Local;
use opening_hours::OpeningHours;

// Opens until 18pm during the week and until 12am the week-end.
const OH: &str = "Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00";

fn main() {
    let oh = OpeningHours::parse(&OH).unwrap();
    let date = Local::now().naive_local();
    println!("Current status is {:?}", oh.state(date).unwrap());
    println!("This will change at {:?}", oh.next_change(date).unwrap());
}
```


Supported features
------------------

### Holidays

A public holiday database is loaded using Python library [workalendar]. You
should refer to Python library for more information on which countries are
supported.

Holidays are not loaded beyond year 2100 to avoid having a huge binary.


### Syntax

If you are only interested in parsing expressions but not on the evaluation or
if you want to build your own evaluation engine, you should probably rely on
the [opening-hours-syntax] crate.



[opening-hours]: https://crates.io/crates/opening-hours
    "Package"

[opening-hours-syntax]: https://crates.io/crates/opening-hours-syntax
    "Syntax Package"

[docs]: https://docs.rs/opening-hours
    "Documentation"

[pypy]: https://pypi.org/project/opening-hours-py
    "Python package"

[codecov]: https://app.codecov.io/gh/remi-dupre/opening-hours-rs
    "Code coverage"

[workalendar]: https://pypi.org/project/workalendar/
    "Worldwide holidays and working days helper and toolkit."
