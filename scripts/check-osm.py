#!/usr/bin/env python3
"""
Fetch examples from taginfo.openstreetmap.org.
"""

import asyncio
from pathlib import Path

import aiohttp
from opening_hours import OpeningHours

CRATE_ROOT = Path(__file__).parent.parent
API_URL = "https://taginfo.openstreetmap.org/api/4/key/values"
PAGE_LENGTH = 999


async def main():
    count_ok = 0
    count_total = 0
    page = 1

    async with aiohttp.ClientSession() as http:
        with open(CRATE_ROOT / "opening-hours" / "data" / "osm_examples.txt", "w") as f:
            while True:
                async with http.get(
                    API_URL,
                    params={
                        "key": "opening_hours",
                        "sortname": "count",
                        "sortorder": "desc",
                        "rp": PAGE_LENGTH,
                        "page": page,
                    },
                ) as resp:
                    content = await resp.json()

                for line in content["data"]:
                    can_parse = True
                    can_eval = False

                    try:
                        oh = OpeningHours(line["value"])
                    except Exception:
                        can_parse = False

                    if can_parse:
                        can_eval = True

                        try:
                            oh.is_open()
                        except Exception:
                            can_eval = False

                    count_total += line["count"]

                    if can_eval:
                        count_ok += line["count"]

                    print(
                        f"{line['count']:06} {can_parse} {can_eval} {line['value']}",
                        file=f,
                    )

                page += 1

                if len(content["data"]) < PAGE_LENGTH:
                    break

    percent = 100 * count_ok / count_total
    print(f"Correct evaluation : {count_ok}/{count_total} ({percent:.2f}%)")


if __name__ == "__main__":
    asyncio.run(main())
