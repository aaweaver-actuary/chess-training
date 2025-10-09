#!/usr/bin/env python3
"""Generate the docs/TYPE_INDEX.md listing all struct and enum definitions."""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
PATTERN = re.compile(r"^\s*(pub(?:\([^)]*\))?\s+)?(struct|enum)\s+([A-Z][A-Za-z0-9_]*)")
IGNORE_DIRS = {".git", "target", "node_modules"}


def should_skip(path: pathlib.Path) -> bool:
    return any(part in IGNORE_DIRS for part in path.parts)


def collect_types() -> list[tuple[str, str, pathlib.Path, int]]:
    types: list[tuple[str, str, pathlib.Path, int]] = []
    seen: dict[str, list[tuple[pathlib.Path, int]]] = {}

    for path in sorted(ROOT.rglob("*.rs")):
        if should_skip(path):
            continue
        try:
            text = path.read_text()
        except UnicodeDecodeError:
            continue
        for idx, line in enumerate(text.splitlines(), start=1):
            stripped = line.lstrip()
            if stripped.startswith("//") or stripped.startswith("/*"):
                continue
            match = PATTERN.match(line)
            if not match:
                continue
            kind = match.group(2)
            name = match.group(3)
            types.append((name, kind, path, idx))
            seen.setdefault(name, []).append((path, idx))

    duplicates = {name: locs for name, locs in seen.items() if len(locs) > 1}
    if duplicates:
        print("Duplicate type definitions found:", file=sys.stderr)
        for name, locations in duplicates.items():
            for path, line in locations:
                rel = path.relative_to(ROOT)
                print(f"  {name}: {rel}#L{line}", file=sys.stderr)
        sys.exit(1)

    types.sort(key=lambda item: item[0])
    return types


def render_table(types: list[tuple[str, str, pathlib.Path, int]]) -> str:
    lines = [
        "# Type Index",
        "",
        "This file lists every `struct` and `enum` defined in the repository.",
        "",
        "| Type | Kind | Location |",
        "| --- | --- | --- |",
    ]
    for name, kind, path, line in types:
        rel = path.relative_to(ROOT).as_posix()
        lines.append(f"| {name} | {kind} | `{rel}#L{line}` |")
    lines.append("")
    lines.append("Generated automatically to support DRY efforts.")
    lines.append("")
    lines.append("Run `python docs/generate_type_index.py` to regenerate.")
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    types = collect_types()
    output = render_table(types)
    output_path = ROOT / "docs" / "TYPE_INDEX.md"
    output_path.write_text(output)
    print(f"Wrote {output_path.relative_to(ROOT)} with {len(types)} entries.")


if __name__ == "__main__":
    main()
