#!/usr/bin/env python3
"""
Ensures the temporary greentic-types compatibility shim is removed
once component-runtime adopts greentic-types >= 0.3.
"""

from __future__ import annotations

import pathlib
import sys
import tomllib


MIN_VERSION = (0, 4, 0)


def parse_version(value: str) -> tuple[int, ...]:
    try:
        return tuple(int(part) for part in value.split("."))
    except ValueError as exc:
        raise SystemExit(f"unable to parse component-runtime version '{value}': {exc}") from exc


def main() -> int:
    lock_path = pathlib.Path("Cargo.lock")
    cargo_toml = pathlib.Path("Cargo.toml")

    if not lock_path.exists():
        print("Cargo.lock not found; skipping compat shim check")
        return 0

    data = tomllib.loads(lock_path.read_text())
    packages = data.get("package", [])
    comp_versions = [pkg["version"] for pkg in packages if pkg.get("name") == "component-runtime"]

    if not comp_versions:
        print("component-runtime is not in Cargo.lock; skipping compat shim check")
        return 0

    # Use the highest version in case multiple appear (e.g., different features)
    version = max(parse_version(ver) for ver in comp_versions)
    compat_present = "greentic-types-compat" in cargo_toml.read_text()

    if version >= MIN_VERSION and compat_present:
        print(
            "component-runtime has reached "
            f"{'.'.join(map(str, version))} but greentic-types-compat is still present.\n"
            "Please drop the compat shim and migrate to greentic-types 0.3+ throughout."
        )
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
