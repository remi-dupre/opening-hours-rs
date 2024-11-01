#!/usr/bin/env -S poetry run python
"""
Display the list of holidays during a given interval of time.
By default all known countries in `workalendar` will be exported.
"""

import asyncio
import sys

import aiohttp
from tap import Tap


API_URL = "https://date.nager.at/api/v3"


class Arg(Tap):
    min_year: int = 2000  # starting year
    max_year: int = 2080  # ending year
    kind: str = "Public"


async def main(arg: Arg, http: aiohttp.ClientSession):
    async with http.get(f"{API_URL}/AvailableCountries") as resp:
        resp.raise_for_status()
        countries = await resp.json()

    for country in countries:
        name = country["name"]
        code = country["countryCode"]

        print(f"Fetching {arg.kind.lower()} holidays for {name}", file=sys.stderr)

        for year in range(arg.min_year, arg.max_year + 1):
            async with http.get(f"{API_URL}/PublicHolidays/{year}/{code}") as resp:
                if not resp.ok:
                    print(f"  - skip year {year}", file=sys.stderr)
                    continue

                resp = await resp.json()

            for day in resp:
                if arg.kind.capitalize() in day["types"]:
                    print(code, day["date"])


async def init():
    arg = Arg().parse_args()

    async with aiohttp.ClientSession() as http:
        await main(arg, http)


if __name__ == "__main__":
    asyncio.run(init())
