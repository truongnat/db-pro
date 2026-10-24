#!/usr/bin/env python3
"""Smoke tests for source ↔ mirror hash verification."""

from __future__ import annotations

import tempfile
from pathlib import Path
from urllib.parse import quote

from knowledge_mirror_utils import (
    encode_mirror_body,
    is_pipeline_memory_artifact,
    mirror_filename,
    mirror_filename_collisions,
    parse_mirror_file,
    sha256_text,
    verify_v2_mirror,
)


def test_leading_newline_roundtrip() -> None:
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        rel = ".cursor/skills/x/LICENSE.txt"
        src = root / rel
        src.parent.mkdir(parents=True)
        raw = b"\nApache License\n"
        src.write_bytes(raw)
        body, truncated, source_sha = encode_mirror_body(rel, raw)
        assert not truncated
        mirror_dir = root / ".agents/memory/entries/migrated/by-source-file"
        mirror_dir.mkdir(parents=True)
        fname = mirror_filename(rel)
        fm = f"""---
source_sha256: "{source_sha}"
mirror_body_sha256: "{sha256_text(body)}"
truncated: false
---"""
        (mirror_dir / fname).write_text(fm.rstrip() + "\n" + body, encoding="utf-8")
        result = verify_v2_mirror(root, rel)
        assert result["status"] == "mirrored_full_verified", result


def test_large_file_full_mirror() -> None:
    raw = b"x" * (512_000 + 10)
    body, truncated, source_sha = encode_mirror_body("big.bin", raw)
    assert not truncated
    assert body == raw.decode("utf-8", errors="replace")
    assert source_sha


def test_parse_preserves_leading_newline() -> None:
    with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as f:
        f.write("---\nkey: v\n---\nhello")
        path = Path(f.name)
    parsed = parse_mirror_file(path)
    path.unlink()
    assert parsed["body"] == "hello"


def test_mirror_filename_percent_encoding() -> None:
    assert mirror_filename("docs/plans/STATUS.md") == quote("docs/plans/STATUS.md", safe="") + ".md"
    assert mirror_filename("foo.json") == quote("foo.json", safe="") + ".md"
    assert mirror_filename("foo.md") != mirror_filename("foo.md.md")
def test_mirror_filename_no_collisions_on_ambiguous_pairs() -> None:
    pairs = [
        ("a/b.md", "a__b.md"),
        ("x/foo", "x/foo.md"),
        ("foo.md", "foo.md.md"),
        ("docs/a.md", "docs/b.md"),
    ]
    for a, b in pairs:
        assert mirror_filename(a) != mirror_filename(b)
    all_paths = [a for a, _ in pairs] + [b for _, b in pairs]
    assert mirror_filename_collisions(all_paths) == {}


def test_mirror_filename_case_folding_detection() -> None:
    from knowledge_mirror_utils import mirror_filename_case_folded_collisions

    # Case-only differences collide on case-insensitive filesystems
    folded = mirror_filename_case_folded_collisions(["Readme.md", "README.md"])
    assert len(folded) == 1
    # Distinct non-case-colliding paths do not
    assert mirror_filename_case_folded_collisions(["docs/a.md", "docs/b.md"]) == {}


def test_pipeline_artifact_detection() -> None:
    assert is_pipeline_memory_artifact(".agents/memory/KNOWLEDGE_SOURCE_INVENTORY.json")
    assert is_pipeline_memory_artifact(".agents/memory/KNOWLEDGE_RECONCILIATION_V2.json")
    assert not is_pipeline_memory_artifact("docs/plans/STATUS.md")
    assert not is_pipeline_memory_artifact(".agents/memory/INDEX.md")


def main() -> int:
    test_leading_newline_roundtrip()
    test_large_file_full_mirror()
    test_mirror_filename_percent_encoding()
    test_mirror_filename_no_collisions_on_ambiguous_pairs()
    test_mirror_filename_case_folding_detection()
    test_pipeline_artifact_detection()
    print("ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
