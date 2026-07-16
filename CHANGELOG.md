# Changelog

## 2.1.5

- Fix: normalization idempotency

## 2.1.4

- Fix: time spans ending with a sun event shouldn't be normalized (there can be
  a day overlap when working with extreme coordinates).

## 2.1.3

- Fix: Display of opening hours that include a holiday offset like `PH -1 days 17:00-02:00`

## 2.1.2

- Fix: variable time at the poles should behave correctly (for example
  "sunrise-sunset" would be always closed in winter and always open in summer).

## 2.1.1

- Fix Rust crate not deploying because of docs static path.

## 2.1.0

The normalization algorithm has been reworked:

- Normalization now focuses on outputing _non-overlapping_ normal rules when possible.
- Added [a documentation][doc-normalize] on normalization's behavior.

[doc-normalize]: https://github.com/remi-dupre/opening-hours-rs/blob/master/opening-hours-syntax/doc/normalize.md

## 2.0.2

### General

- Update dependencies (noticeably PyO3 CVE)

## 2.0.0

### General

- **(breaking)** Intervals from iterators will now return a unique comment (an
  empty string if there is no comment).
- **(breaking)** `next_change()` can now return a date where the facility remains
  open or remains closed if the comment changes.
- **(breaking)** `state()` now returns the current comment together with the
  rule kind.
- **(breaking)** Year and week ranges cannot be defined in inverted order,
  which is similarly to the JS library's behavior.
- Syntax: handle week days in month selectors (eg. "Jan Su[-1]-Jul Mo[1]")
- Fix: expressions formatting to invalid expression
  - time spans with repetition (eg. "12:00-14:00/01:30")
  - variable times with offset (eg. "(sunrise-00:10)-(sunset+01:15)")
- The parser won't log any warnings anymore, it is however now possible to
  register a callback to handle more kind of warnings.
- Syntax: Literals are no longer case sensitive (emits a warning).

### Python

- Added `max_interval_days` parameter that allows to enable an evaluation
  optimisation at the cost of precision.

### Rust

- Add `OpeningHours::get_context(&self)`.
- Infaillible parser: parsing any call is garanteed to be panic-free. Added new
  class of implementation issues that would prompt you to open a Github issue.
- Deprecate `OpeningHours::parse`, use `std::str::FromStr` instead.
- Syntax: remove log feature.

## 1.4.0

- Regional holidays will now be considered in unknown status.
- Update holidays database from nagger. Support for new contries: Bangladesh
  and Uganda.

## 1.3.1

### General

- Fix: year ranges of a single value with a step were formatted
  into an invalid expression.

## 1.3.0

### General

- Fix: expression stringification was not idempotent in some cases

### Rust

- opening-hours-syntax: support for no-std environment (ft. @hosseinpro)

## 1.2.1

### General

- Fix: normalization used a normal rule operator instead of additional rule
  operator in some case
  (Issue [#97](https://github.com/remi-dupre/opening-hours-rs/issues/97))
- Fix: normalization prioritize full time ranges for a more natural result.
  (Issue [#98](https://github.com/remi-dupre/opening-hours-rs/issues/98))
- Chore: Update dependencies

## 1.2.0

### General

- Fix [#91](https://github.com/remi-dupre/opening-hours-rs/issues/91):
  normalization does not prefix normal separators with a space anymore.
- Update holidays database from nagger. Support for new contries: DR Congo,
  Congo, Ghana Seychelles and Türkiye.

### Python

- Fix [#92](https://github.com/remi-dupre/opening-hours-rs/issues/92):
  missing wheels on Linux
- Fix [#90](https://github.com/remi-dupre/opening-hours-rs/issues/90):
  missing README to PyPI
- Bump Maturin to 1.12 (build system) & update generated CI

## 1.1.6

### General

- Fix [#88](https://github.com/remi-dupre/opening-hours-rs/issues/88):
  allow space inside of time blocks.
- Chore (CI): bump x86 runners for MacOS to version 14
- Chore: update dependencies

## 1.1.5

### General

- Fix generated calendar not being platform agnostic.

## 1.1.4

### Rust

- Use `docsrs` cfg to enable `doc_cfg` feature instead of checking for rustc nightly channel
- Update dependencies

### Python

- Only build for Python >= 3.10
- Update dependencies
- Build on Ubuntu 24.04 LTS (from 22.04)

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
