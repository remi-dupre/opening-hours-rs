🐍 Python bindings for [OSM Opening Hours](https://github.com/remi-dupre/opening-hours-rs)
==========================================

[![PyPI](https://img.shields.io/pypi/v/opening-hours-py)][pypi]
[![Doc](https://img.shields.io/badge/doc-pdoc-blue)][docs]
[![PyPI - Downloads](https://img.shields.io/pypi/dm/opening-hours-py)][pypi]


Usage
-----

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
```

The API is very similar to Rust API but you can find a Python specific
documentation [here](https://remi-dupre.github.io/opening-hours-rs/opening_hours.html).


Developement
-------------

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


[pypi]: https://pypi.org/project/opening-hours-py/
[docs]: https://remi-dupre.github.io/opening-hours-rs/opening_hours.html
[python-versions]: https://devguide.python.org/versions/#supported-versions
