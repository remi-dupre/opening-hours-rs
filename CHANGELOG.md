# Changelog

## 1.1.4

### Rust

- Use `docsrs` cfg to enable `doc_auto_cfg` feature instead of checking for rustc nightly channel

## 1.1.3

### General

- Fix crashes when a repetition is defined in a time span (eg.
  "10:00-18:00/01:30")
- Update dependencies (PyO3 0.24.1)

## 1.1.2

### Rust

- Switch from `sunrise-next` dependency back to `sunrise` as all changes
  have been upstreamed.

## 1.1.1

### Rust

- Upgrade to edition 2024

## 1.1.0

### General

- Allow to normalize "canonical" expressions (expressions expressed as simple
  intervals over each dimension).
- Weird expressions equivalent to "24/7" should generally be evaluated faster.
- Fixed a lot of bugs. This comes from the fuzzer being super happy of the
  addition of a normalization which acts as a sort of concurrent implementation
  of the evaluation rules.

### Rust

- Add `approx_bound_interval_size` option to context to allow optimizing calls
  to `next_change` over long periods of time.

### Fixes

- NaN values are now ignored in coordinates inputs.
- Empty expressions are no longer allowed.
- Monthday "0" is no no longer allowed.

## 1.0.3

### Python

- stub: fix variants casing for `State`

## 1.0.2

### Python

- Add auto-generated Python stub file.

## 1.0.0

That's not really a huge milestone, but:

- Every "obviously missing things" that I had in mind are implemented now.
- The API proved itself to be quite stable.

### General

- Add Easter support

## 0.11.1

### Rust

- More robust week selector

## 0.11.0

### General

- General reorganisation of modules

### Rust

- Coordinates validation
- Add fuzzing corpus

## 0.10.1

### Rust

- Fix missing items in documentation

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
