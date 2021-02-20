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

import httpx
import toml
from semantic_version import Version


PYPI_META_URL = "https://pypi.org/pypi/opening-hours-py/json"
CRATE_META_URL = "https://crates.io/api/v1/crates/opening-hours"


async def get_pypi_version():
    async with httpx.AsyncClient() as http:
        res = await http.get(PYPI_META_URL)

    res.raise_for_status()
    return Version(res.json()["info"]["version"])


async def get_crate_version():
    async with httpx.AsyncClient() as http:
        res = await http.get(CRATE_META_URL)

    res.raise_for_status()
    return Version(res.json()["crate"]["newest_version"])


async def main():
    return_status = 0

    root = Path(__file__).parent.parent
    rs_version = toml.load(root / "Cargo.toml")["package"]["version"]
    py_version = toml.load(root / "python/Cargo.toml")["package"]["version"]
    pt_version = toml.load(root / "python/pyproject.toml")["tool"]["poetry"]["version"]

    print("Checking local packages:")
    print(" - Rust crate:", rs_version)
    print(" - Python crate:", py_version)
    print(" - Python package:", pt_version)

    if not rs_version == py_version == pt_version:
        print(r"/!\ Packages versions don't match")
        return_status = 1

    command = ["git", "branch", "--show-current"]
    result = subprocess.run(command, stdout=subprocess.PIPE, check=True)
    branch = result.stdout.decode().strip()

    if branch != "master":
        local_version = Version(rs_version)
        pypi_version, crate_version = await asyncio.gather(
            get_pypi_version(), get_crate_version()
        )

        print(f"Current branch is {branch}, checking remote packages:")
        print(" - crates.io version:", crate_version)
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
