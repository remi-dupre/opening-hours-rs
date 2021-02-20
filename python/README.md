Python bindings for [OSM Opening Hours](https://github.com/remi-dupre/opening-hours-rs)
=======================================

[![PyPI](https://img.shields.io/pypi/v/opening-hours-py)](https://pypi.org/project/opening-hours-py/)
[![Doc](https://img.shields.io/badge/doc-pdoc-blue)](https://remi-dupre.github.io/opening-hours-rs/opening_hours.html)


Usage
-----

Install `opening-hours-py` from PyPI, for example using pip:

```bash
pip install --user opening-hours-rs
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


Developpement
-------------

To build the library by yourself you will require a recent version of Rust,
[`rustup`](https://www.rust-lang.org/tools/install) is usually the recommanded
tool to manage the installation.

Then you can use poetry to install Python dependancies and run `maturin` (the
building tool used to create the binding) from a virtualenv.

```bash
git clone https://github.com/remi-dupre/opening-hours-rs.git
cd opening-hours-rs/python

# Install Python dependancies
poetry install

# Enter the virtualenv
poetry shell

# Build developpement bindings, add `--release` for an optimized version
maturin develop

# Now the library is available as long as you don't leave the virtualenv
python
>>> from opening_hours import OpeningHours
>>> oh = OpeningHours("24/7")
>>> oh.state()
"open"
```
