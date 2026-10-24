#!/usr/bin/env python3
"""Fail if two repo text sources map to the same mirror_filename."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from _load_build_inventory import load as load_build  # noqa: E402
from knowledge_mirror_utils import (  # noqa: E402
    assert_mirror_filename_unique,
    mirror_filename_case_folded_collisions,
    mirror_filename_collisions,
)


def main() -> int:
    paths = [r["path"] for r in load_build().scan_repo()]
    col = mirror_filename_collisions(paths)
    if col:
        print(json.dumps({"collisions": col}, indent=2), file=sys.stderr)
        return 1
    folded = mirror_filename_case_folded_collisions(paths)
    if folded:
        print(json.dumps({"case_folded_collisions": folded}, indent=2), file=sys.stderr)
        return 1
    assert_mirror_filename_unique(paths)
    print(json.dumps({"repo_sources": len(paths), "collisions": 0, "case_folded_collisions": 0}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
