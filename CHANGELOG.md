# Changelog

## 0.10.0

### Rust

- Rust 1.83
- Support localization (timezone & coords)
- add default feature _log_
- add optional feature _auto-country_
- add optional feature _auto-timezone_

### Python

- Add the `opening_hours.State` type.
- Updated to latest maturin's workflow, which should ship precompiled binaries
  for more older Python version in the future.
- Support localization (timezone & coords)
- Add exception types `ParserError` and `UnknownCountryError`

## 0.9.1

### Fixes

- Fix [#56](https://github.com/remi-dupre/opening-hours-rs/issues/56):
  expressions with no date filter (eg. `00:30-05:30`) may be considered as
  always closed.

## 0.9.0

### General

- Holidays database from [nager.date](https://date.nager.at/).
- Some support for public holidays.
- Replace all panicking functions with faillible ones.

### Rust

- `OpeningHours` now implements `FromStr`.
- `CompactCalendar` is no longer bounded.
- Added `Context`, which will later be extended to handle localization info.
- Added module `country`.
- Better documentation converage.

### Python

- Got rid of `unsafe` used in `OpeningHours.intervals` implementation.
- The iterator returned by `OpeningHours.intervals` can be moved between
  threads.

## 0.8.3

### Fixes

- Fix [#52](https://github.com/remi-dupre/opening-hours-rs/pull/52): intervals
  were stopping at midnight before the last day.

## 0.8.2

### Fixes

- Python's Linux binary build were not uploading

## 0.8.1

### Fixes

- Rust crate couldn't publish

## 0.8.0

### General

- Emit some logs when parsing unsupported syntax.
- Basic support for stringifying `OpeningHours`

### Python

- Implement `__repr__`, `__str__`, `__hash__` and `__eq__`
- Upgrade to PyO3 0.22 (from 0.19) which natively supports datetime conversions

### Fixes

- Most crashing edge cases have been removed (through `.expect()` removal and fuzzing)
- Monthday & Time ambiguity has been fixed for parser (eg. "Oct 12:00-24:00")
