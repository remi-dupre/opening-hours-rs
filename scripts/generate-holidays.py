#!/usr/bin/env -S poetry run python
"""
Display the list of holidays during a given interval of time.
By default all known countries in `workalendar` will be exported.
"""

import asyncio
import sys
from dataclasses import dataclass
from pathlib import Path

import aiohttp
from tap import Tap


API_URL = "https://date.nager.at/api/v3"

CRATE_ROOT = Path(__file__).parent.parent
CRATE_ROOT_SRC = CRATE_ROOT / "opening-hours"

FILE_PUBLIC_HOLIDAYS = CRATE_ROOT_SRC / "data" / "holidays_public.txt"
FILE_SCHOOL_HOLIDAYS = CRATE_ROOT_SRC / "data" / "holidays_school.txt"
FILE_ENUM_TEMPLATE = CRATE_ROOT_SRC / "data" / "templates" / "countries.rs.jinja"
FILE_ENUM_OUTPUT = CRATE_ROOT_SRC / "src" / "country" / "generated.rs"


class Dates(Tap):
    command: str = "dates"
    min_year: int = 2000  # starting year
    max_year: int = 2080  # ending year


class Enum(Tap):
    command: str = "enum"


class Arg(Tap):
    def configure(self):
        self.add_subparser("dates", Dates)
        self.add_subparser("enum", Enum)


@dataclass
class Country:
    name: str
    iso_code: str


async def load_countries(http: aiohttp.ClientSession) -> list[Country]:
    async with http.get(f"{API_URL}/AvailableCountries") as resp:
        resp.raise_for_status()
        countries = await resp.json()

    return [
        Country(name=country["name"], iso_code=country["countryCode"])
        for country in countries
    ]


async def load_dates(
    arg: Arg,
    http: aiohttp.ClientSession,
    countries: list[Country],
    kind: str,
    output_file: Path,
):
    file = open(output_file, "w")

    for country in countries:
        print(
            f"Fetching {arg.kind.lower()} holidays for {country.name}", file=sys.stderr
        )

        for year in range(arg.min_year, arg.max_year + 1):
            async with http.get(
                f"{API_URL}/PublicHolidays/{year}/{country.iso_code}"
            ) as resp:
                if not resp.ok:
                    print(f"  - skip year {year}", file=sys.stderr)
                    continue

                resp = await resp.json()

            for day in resp:
                if arg.kind.capitalize() in day["types"]:
                    print(country.iso_code, day["date"], file=file)

    file.close()


def generate_enum(countries: list[Country]):
    from jinja2 import Template

    with open(FILE_ENUM_TEMPLATE) as f:
        template = Template(f.read())

    source = template.render(countries=countries)

    with open(FILE_ENUM_OUTPUT, "w") as f:
        f.write(source)


async def main(arg: Arg, http: aiohttp.ClientSession):
    countries = await load_countries(http)

    match arg.command:
        case "dates":
            for kind, path in [
                ("Public", FILE_PUBLIC_HOLIDAYS),
                ("School", FILE_SCHOOL_HOLIDAYS),
            ]:
                await load_dates(arg, http, countries, kind, path)
        case "enum":
            generate_enum(countries)


async def init():
    arg = Arg().parse_args()

    async with aiohttp.ClientSession() as http:
        await main(arg, http)


if __name__ == "__main__":
    asyncio.run(init())
