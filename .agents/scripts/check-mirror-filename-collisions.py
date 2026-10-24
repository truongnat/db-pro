#!/usr/bin/env python3
"""Fail if two repo text sources map to the same mirror_filename."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _load_build_inventory import load as load_build  # noqa: E402
from knowledge_mirror_utils import mirror_filename_collisions  # noqa: E402


def main() -> int:
    paths = [r["path"] for r in load_build().scan_repo()]
    col = mirror_filename_collisions(paths)
    if col:
        print(json.dumps({"collisions": col}, indent=2), file=sys.stderr)
        return 1
    print(json.dumps({"repo_sources": len(paths), "collisions": 0}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
