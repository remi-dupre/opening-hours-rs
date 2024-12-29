# 🐍 Python bindings for [OSM Opening Hours](https://github.com/remi-dupre/opening-hours-rs)

[![PyPI](https://img.shields.io/pypi/v/opening-hours-py)][pypi]
[![Doc](https://img.shields.io/badge/doc-pdoc-blue)][docs]
[![PyPI - Downloads](https://img.shields.io/pypi/dm/opening-hours-py)][pypi]

## Usage

The pre-compiled package is published for Python 3.9 and above and new releases
will adapt to [officially supported Python versions][python-versions].

If you want to install this library with older version of Python, **you will
need the Rust toolchain** (`rustc` and `cargo`).

Install `opening-hours-py` from PyPI, for example using pip:

```bash
pip install --user opening-hours-py
```

Then, the main object that you will interact with will be `OpeningHours`:

```python
from opening_hours import OpeningHours

oh = OpeningHours("Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00")
print("Current status is", oh.state())
print("This will change at", oh.next_change())

# You can also attach a timezone to your expression. If you use timezone-aware
# dates, they will be converted to local time before any computation is done.
from zoneinfo import ZoneInfo
oh = OpeningHours("Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00", timezone=ZoneInfo("Europe/Paris"))

# The timezone can also be infered with coordinates
oh = OpeningHours("Mo-Fr 10:00-18:00; Sa-Su 10:00-12:00", coords=(48.8535, 2.34839))
```

The API is very similar to Rust API but you can find a Python specific
documentation [here](https://remi-dupre.github.io/opening-hours-rs/opening_hours.html).

## Features

- 📝 Parsing for [OSM opening hours][grammar]
- 🧮 Evaluation of state and next change
- ⏳ Lazy infinite iterator
- 🌅 Accurate sun events
- 📅 Embedded public holidays database for many countries (from [nagger])
- 🌍 Timezone support
- 🔥 Fast and memory-safe implementation using Rust

## Development

To build the library by yourself you will require a recent version of Rust,
[`rustup`](https://www.rust-lang.org/tools/install) is usually the recommended
tool to manage the installation.

Then you can use poetry to install Python dependencies and run `maturin` (the
building tool used to create the bindings) from a virtualenv.

```bash
$ git clone https://github.com/remi-dupre/opening-hours-rs.git
$ cd opening-hours-rs

# Install Python dependancies
$ poetry install

# Enter the virtualenv
$ poetry shell

# Build developpement bindings, add `--release` for an optimized version
$ maturin develop

# Now the library is available as long as you don't leave the virtualenv
$ python
>>> from opening_hours import OpeningHours
>>> oh = OpeningHours("24/7")
>>> oh.state()
"open"
```

[docs]: https://remi-dupre.github.io/opening-hours-rs/opening_hours.html "Generated documentation"
[grammar]: https://wiki.openstreetmap.org/wiki/Key:opening_hours/specification "OSM specification for opening hours"
[nager]: https://date.nager.at/api/v3 "Worldwide holidays (REST API)"
[pypi]: https://pypi.org/project/opening-hours-py/ "PyPI page"
[python-versions]: https://devguide.python.org/versions/#supported- "Python release cycle"
