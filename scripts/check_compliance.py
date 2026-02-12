#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    print(f"compliance check failed: {message}")
    sys.exit(1)


def main() -> None:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    if 'license = "AGPL-3.0-or-later"' not in cargo:
        fail("workspace license must be AGPL-3.0-or-later")

    notices_path = ROOT / "THIRD_PARTY_NOTICES.md"
    if not notices_path.exists():
        fail("THIRD_PARTY_NOTICES.md is missing")
    notices_text = notices_path.read_text(encoding="utf-8")

    manifest_path = ROOT / "provenance" / "imported_code.json"
    if not manifest_path.exists():
        fail("provenance/imported_code.json is missing")

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"invalid provenance manifest JSON: {exc}")

    imports = manifest.get("imports")
    if not isinstance(imports, list):
        fail("provenance manifest must contain an 'imports' list")

    for idx, entry in enumerate(imports):
        if not isinstance(entry, dict):
            fail(f"imports[{idx}] must be an object")

        required = [
            "local_path",
            "origin_project",
            "origin_file",
            "origin_license",
            "reuse_type",
            "recorded_at",
        ]
        for key in required:
            value = entry.get(key)
            if not isinstance(value, str) or not value.strip():
                fail(f"imports[{idx}].{key} must be a non-empty string")

        local_path = ROOT / entry["local_path"]
        if not local_path.exists():
            fail(f"imports[{idx}] references missing file: {entry['local_path']}")

        if entry["local_path"] not in notices_text:
            fail(
                "THIRD_PARTY_NOTICES.md must mention imported file "
                f"{entry['local_path']}"
            )

        if not re.match(r"^\d{4}-\d{2}-\d{2}$", entry["recorded_at"]):
            fail(f"imports[{idx}].recorded_at must use YYYY-MM-DD")

    print("compliance check passed")


if __name__ == "__main__":
    main()
