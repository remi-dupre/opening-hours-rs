#!/usr/bin/env -S pipenv run python
import sys
from typing import List

from tap import Tap
from workalendar.registry import registry
from workalendar.exceptions import CalendarError


class Arg(Tap):
    min_year: int = 2020  # starting year
    max_year: int = 2050  # ending year
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
                print(country, date.isoformat())
        except Exception:
            pass
