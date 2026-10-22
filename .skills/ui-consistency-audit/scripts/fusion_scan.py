#!/usr/bin/env python3
"""Runtime "fused UI" scanner for DB Pro capture audits.

Input: the `DB_PRO_INSPECTOR_DUMP` stderr logs produced by
.skills/ui-consistency-audit/scripts/capture_ui_matrix.sh --audit.

It measures *rendered* widget geometry, so unlike a grep-based audit it counts
only collisions that actually appear on screen: two sibling widgets that share
a parent Ui and a row, with no visual gap between them.

Usage:
    python3 fusion_scan.py <dir-with-*-dump.log> [--gap 2.5] [--json]
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import sys

WID_RE = re.compile(
    r"^WID (?P<id>\S+) layer=(?P<layer>\S+) (?P<layerid>\S+) "
    r"parent=(?P<parent>\S+) "
    r"rect=GeometryRect \{ min_x: (?P<x0>[-\d.]+), min_y: (?P<y0>[-\d.]+), "
    r"max_x: (?P<x1>[-\d.]+), max_y: (?P<y1>[-\d.]+) \} "
    r"click=(?P<click>\w+) enabled=(?P<enabled>\w+)"
)

# A gap below this is not a gap: the two widgets read as one block.
DEFAULT_GAP_PX = 2.5
# Below this size a rect is a layout artifact (zero-size interact rect, caret).
MIN_SIDE_PX = 3.0
# Siblings must overlap this much of the shorter height to be "the same row".
MIN_Y_OVERLAP = 0.6
# Side panels share their edges by design (activity rail | sidebar | central).
# Two rects this tall are containers, not row controls: reported separately.
CONTAINER_MIN_HEIGHT_PX = 400.0


def parse_log(path: str):
    widgets = {}
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            m = WID_RE.match(line)
            if not m:
                continue
            g = m.groupdict()
            key = (g["layerid"], g["parent"], g["id"])
            widgets[key] = {
                "id": g["id"],
                "layer": g["layerid"],
                "parent": g["parent"],
                "x0": float(g["x0"]),
                "y0": float(g["y0"]),
                "x1": float(g["x1"]),
                "y1": float(g["y1"]),
                "click": g["click"] == "true",
                "enabled": g["enabled"] == "true",
            }
    # The dump repeats per frame; keep the last (settled) geometry per widget.
    return list(widgets.values())


def fused_pairs(widgets, gap_px: float):
    """Return (fused row pairs, adjacent container pairs)."""

    def is_panel(w):
        return (w["y1"] - w["y0"]) >= CONTAINER_MIN_HEIGHT_PX

    visible = [
        w
        for w in widgets
        if w["enabled"]
        and (w["x1"] - w["x0"]) >= MIN_SIDE_PX
        and (w["y1"] - w["y0"]) >= MIN_SIDE_PX
    ]
    groups = {}
    for w in visible:
        groups.setdefault((w["layer"], w["parent"]), []).append(w)

    findings = []
    containers = []
    for group in groups.values():
        group.sort(key=lambda w: w["x0"])
        for left, right in zip(group, group[1:]):
            gap = right["x0"] - left["x1"]
            if not 0.0 <= gap < gap_px:
                continue
            lo = max(left["y0"], right["y0"])
            hi = min(left["y1"], right["y1"])
            shortest = min(left["y1"] - left["y0"], right["y1"] - right["y0"])
            if shortest <= 0 or (hi - lo) / shortest < MIN_Y_OVERLAP:
                continue
            # A full-bleed container behind a row is layering, not fusion.
            if (left["x1"] - left["x0"]) >= 1000 or (right["x1"] - right["x0"]) >= 1000:
                continue
            pair = {"gap_px": round(gap, 2), "left": left, "right": right}
            # Adjacent panels share their edge on purpose; count them apart so
            # the headline number stays honest either way.
            (containers if is_panel(left) and is_panel(right) else findings).append(pair)
    return findings, containers


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("target")
    ap.add_argument("--gap", type=float, default=DEFAULT_GAP_PX)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    logs = sorted(glob.glob(os.path.join(args.target, "*.log")))
    if not logs:
        print(f"no *.log under {args.target}", file=sys.stderr)
        return 2

    report = []
    for log in logs:
        surface = os.path.basename(log).replace(".log", "")
        findings, containers = fused_pairs(parse_log(log), args.gap)
        report.append(
            {
                "surface": surface,
                "count": len(findings),
                "container_pairs": len(containers),
                "findings": findings,
            }
        )

    if args.json:
        print(json.dumps(report, indent=1))
    else:
        total = sum(r["count"] for r in report)
        panels = sum(r["container_pairs"] for r in report)
        print(
            f"gap threshold: <{args.gap}px   surfaces: {len(report)}   "
            f"fused pairs: {total}   (excluded adjacent-panel edge pairs: {panels})\n"
        )
        for r in report:
            if not r["count"]:
                continue
            print(f"{r['surface']}: {r['count']}")
            for f in r["findings"]:
                l, rr = f["left"], f["right"]
                print(
                    f"  {f['gap_px']:>5}px  {l['id']}[{l['x0']:.0f},{l['y0']:.0f}-"
                    f"{l['x1']:.0f},{l['y1']:.0f}] x {rr['id']}[{rr['x0']:.0f},"
                    f"{rr['y0']:.0f}-{rr['x1']:.0f},{rr['y1']:.0f}] parent={l['parent']}"
                )
    return 0


if __name__ == "__main__":
    sys.exit(main())
