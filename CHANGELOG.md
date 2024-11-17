# Changelog

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
