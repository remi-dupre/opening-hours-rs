#!/usr/bin/env -S pipenv run python
"""
Display the list of holidays during a given interval of time.
By default all known countries in `workalendar` will be exported.
"""
from typing import List

from tap import Tap
from workalendar.registry import registry
from workalendar.exceptions import CalendarError


class Arg(Tap):
    min_year: int = 2000  # starting year
    max_year: int = 2100  # ending year
    regions: List[str] = None  # list of regions to import events from


args = Arg().parse_args()


if args.regions is None:
    cals = registry.get_calendars(include_subregions=True)
else:
    cals = registry.get_calendars([reg.upper() for reg in args.regions])


for year in range(args.min_year, args.max_year + 1):
    for country, cal in cals.items():
        try:
            for date, _name in cal().holidays(year):
                # Sometime the lib gives an overlap
                if args.min_year <= date.year <= args.max_year:
                    print(country, date.isoformat())
        except (CalendarError, KeyError, NotImplementedError, ValueError):
            pass
