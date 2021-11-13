#!/usr/bin/env python3
"""
Run all doctests for `opening_hours` module.
"""
import doctest
import opening_hours
from opening_hours import OpeningHours


if __name__ == "__main__":
    opening_hours.__test__ = dict(OpeningHours.__dict__) | {
        "OpeningHours": OpeningHours
    }

    globs = {
        "opening_hours": opening_hours,
        "OpeningHours": OpeningHours,
    }

    doctest.testmod(
        opening_hours, globs=globs, optionflags=doctest.ELLIPSIS, verbose=True
    )
