#!/usr/bin/env python3
"""
Ensure the version of the repository is consistent, precisely:
  1. The version of the repo must be the same for root crate and Python's crate.
  2. If this runs outside of the master branch, the version must be greater
     than the one uploaded to crates.io and pypi.org.
"""

import asyncio
import subprocess
import sys
from pathlib import Path

import aiohttp
import toml
from semantic_version import Version


PYPI_META_URL = "https://pypi.org/pypi/opening-hours-py/json"
CRATE_META_URL = "https://crates.io/api/v1/crates/opening-hours"
SYNTAX_META_URL = "https://crates.io/api/v1/crates/opening-hours-syntax"
CCAL_META_URL = "https://crates.io/api/v1/crates/compact-calendar"


async def get_pypi_version():
    async with aiohttp.ClientSession() as http:
        res = await http.get(PYPI_META_URL)
        res.raise_for_status()
        res = await res.json()

    return Version(res["info"]["version"])


async def get_crate_version(url=CRATE_META_URL):
    async with aiohttp.ClientSession() as http:
        res = await http.get(url)
        res.raise_for_status()
        res = await res.json()

    return Version(res["crate"]["newest_version"])


async def main():
    return_status = 0

    rt = Path(__file__).parent.parent
    rs_version = toml.load(rt / "Cargo.toml")["package"]["version"]
    sy_version = toml.load(rt / "opening-hours-syntax/Cargo.toml")["package"]["version"]
    cc_version = toml.load(rt / "compact-calendar/Cargo.toml")["package"]["version"]
    py_version = toml.load(rt / "opening-hours-py/Cargo.toml")["package"]["version"]
    pt_version = toml.load(rt / "pyproject.toml")["tool"]["poetry"]["version"]

    print("Checking local packages:")
    print(" - Rust crate:", rs_version)
    print(" - Syntax crate", sy_version)
    print(" - Compact Calendar crate", cc_version)
    print(" - Python crate:", py_version)
    print(" - Python package:", pt_version)

    if not rs_version == sy_version == cc_version == py_version == pt_version:
        print(r"/!\ Packages versions don't match")
        return_status = 1

    command = ["git", "branch", "--show-current"]
    result = subprocess.run(command, stdout=subprocess.PIPE, check=True)
    branch = result.stdout.decode().strip()

    if branch != "master":
        local_version = Version(rs_version)

        (
            pypi_version,
            crate_version,
            syntax_version,
            ccal_version,
        ) = await asyncio.gather(
            get_pypi_version(),
            get_crate_version(),
            get_crate_version(SYNTAX_META_URL),
            get_crate_version(CCAL_META_URL),
        )

        print(f"Current branch is {branch}, checking remote packages:")
        print(" - opening-hours version:", crate_version)
        print(" - opening-hours-syntax version:", syntax_version)
        print(" - compact-calendar version:", ccal_version)
        print(" - PyPI version:", pypi_version)

        if pypi_version >= local_version:
            print(r"/!\ Version isn't higher than PyPI package")
            return_status = 2

        if crate_version >= local_version:
            print(r"/!\ Version isn't higher than crates.io package")
            return_status = 2

    return return_status


if __name__ == "__main__":
    status = asyncio.run(main())
    sys.exit(status)
