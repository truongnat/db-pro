#!/usr/bin/env python3
"""Smoke tests for source ↔ mirror hash verification."""

from __future__ import annotations

import tempfile
from pathlib import Path

from knowledge_mirror_utils import (
    encode_mirror_body,
    is_pipeline_memory_artifact,
    mirror_filename,
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
        fname = ".cursor__skills__x__LICENSE.txt.md"
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


def test_mirror_filename_encoding() -> None:
    assert mirror_filename("docs/plans/STATUS.md") == "docs__plans__STATUS.md"
    assert mirror_filename("foo.json") == "foo.json.md"
    assert mirror_filename("weird.md") == "weird.md"
    assert mirror_filename("weird.md.md") == "weird.md.md"
    assert mirror_filename("weird.md") != mirror_filename("weird.md.md")


def test_mirror_filename_collisions_detected() -> None:
    from knowledge_mirror_utils import mirror_filename_collisions

    # Encoding quirk: extensionless file vs sibling `.md` share one mirror name.
    col = mirror_filename_collisions(["x/foo", "x/foo.md"])
    assert list(col.keys()) == ["x__foo.md"]
    assert set(col["x__foo.md"]) == {"x/foo", "x/foo.md"}
    assert mirror_filename_collisions(["docs/a.md", "docs/b.md"]) == {}


def test_pipeline_artifact_detection() -> None:
    assert is_pipeline_memory_artifact(".agents/memory/KNOWLEDGE_SOURCE_INVENTORY.json")
    assert is_pipeline_memory_artifact(".agents/memory/KNOWLEDGE_RECONCILIATION_V2.json")
    assert not is_pipeline_memory_artifact("docs/plans/STATUS.md")
    assert not is_pipeline_memory_artifact(".agents/memory/INDEX.md")


def main() -> int:
    test_leading_newline_roundtrip()
    test_large_file_full_mirror()
    test_parse_preserves_leading_newline()
    test_mirror_filename_encoding()
    test_mirror_filename_collisions_detected()
    test_pipeline_artifact_detection()
    print("ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
