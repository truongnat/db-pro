#!/usr/bin/env python3
"""Remove legacy mirror files that do not match any repo source's mirror_filename.

Does not change mirror_filename encoding — only deletes orphans on disk (e.g. old
*.md.md duplicates from mistaken double-suffix mirrors).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MIRROR_BY_FILE = ROOT / ".agents/memory/entries/migrated/by-source-file"

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _load_build_inventory import load as load_build  # noqa: E402
from knowledge_mirror_utils import mirror_filename  # noqa: E402


def main() -> int:
    expected = {mirror_filename(r["path"]) for r in load_build().scan_repo()}
    if not MIRROR_BY_FILE.is_dir():
        print(json.dumps({"removed": 0, "note": "mirror dir missing"}))
        return 0

    removed: list[str] = []
    for p in sorted(MIRROR_BY_FILE.iterdir()):
        if not p.is_file():
            continue
        if p.name not in expected:
            p.unlink()
            removed.append(p.name)

    print(json.dumps({"removed": len(removed), "sample": removed[:20]}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
