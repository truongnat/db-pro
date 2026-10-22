#!/usr/bin/env python3
"""Static UI consistency scanner for the DB Pro egui codebase.

Inventories Select call sites, flags contract violations in shared components,
and cross-references capture hooks. Report-only by default; `--strict` exits 1
when a contract violation (not a style preference) is found.

This file is part of the self-improving loop described in SKILL.md: when a new
finding generalizes and is statically detectable, add a rule here.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
APP_SRC = ROOT / "crates" / "app" / "src"
UI_SRC = ROOT / "crates" / "ui" / "src"
COMPONENTS = UI_SRC / "components"
CAPTURE_RS = ROOT / "crates" / "native-app" / "src" / "capture.rs"

SELECT_RE = re.compile(r"(?:\bSelect|components::Select)::new\(")
THEME_DEFAULT_RE = re.compile(r"theme:\s*DbProTheme::default\(\)")
POPUP_OPEN_RE = re.compile(r"Popup::open_id\(\s*[^,]+,\s*(?:egui::)?Id::new\(\"([^\"]+)\"\)")

SKIP_PARTS = ("/tests/", "/component_gallery/")
SKIP_NAMES = ("tests.rs",)


@dataclass
class CallSite:
    path: Path
    line: int
    salt: str
    size: str = "default"
    variant: str = "default"
    width: str = "available"
    theme: bool = False
    label: bool = False
    hand_rolled_label: bool = False
    contexts: list[str] = field(default_factory=list)
    flags: list[str] = field(default_factory=list)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def line_of(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def statement_from(text: str, pos: int) -> str:
    """Return the expression starting at pos up to the top-level semicolon.

    Tracks (), [], {}, strings, chars and comments so semicolons inside
    closures or format strings do not terminate the statement.
    """
    depth = 0
    i = pos
    quote = None
    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if quote:
            if ch == "\\":
                i += 2
                continue
            if ch == quote:
                quote = None
        elif ch in ('"', "'"):
            quote = ch
        elif ch == "/" and nxt == "/":
            end = text.find("\n", i)
            i = len(text) if end == -1 else end
            continue
        elif ch == "/" and nxt == "*":
            end = text.find("*/", i + 2)
            i = len(text) if end == -1 else end + 2
            continue
        elif ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == ";" and depth == 0:
            return text[pos : i + 1]
        i += 1
    return text[pos:]


def call_arg(expr: str, name: str) -> str | None:
    """Return the raw argument text of the first `.name(...)` call."""
    needle = f".{name}("
    idx = expr.find(needle)
    if idx == -1:
        return None
    start = idx + len(needle)
    depth = 1
    i = start
    quote = None
    while i < len(expr):
        ch = expr[i]
        if quote:
            if ch == "\\":
                i += 2
                continue
            if ch == quote:
                quote = None
        elif ch in ('"', "'"):
            quote = ch
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return expr[start:i].strip()
        i += 1
    return None


def salt_of(expr: str) -> str:
    m = re.search(r"::new\(\s*([^,]+),", expr)
    if not m:
        return "?"
    raw = m.group(1).strip()
    if raw.startswith("format!("):
        fmt = re.search(r'format!\(\s*"([^"]+)"', raw)
        return fmt.group(1) if fmt else "format!"
    raw = raw.lstrip("&").strip()
    return raw.strip('"') if raw.startswith('"') else raw


def context_flags(text: str, pos: int) -> list[str]:
    line_start = text.rfind("\n", 0, pos)
    window_start = text.rfind("\n", 0, max(0, line_start - 1))
    # Look back ~30 lines for the parent layout.
    start = pos
    for _ in range(30):
        prev = text.rfind("\n", 0, start)
        if prev == -1:
            break
        start = prev
    window = text[start:pos]
    flags = []
    if re.search(r"right_to_left", window):
        flags.append("rtl-parent")
    if re.search(r"horizontal(?:_wrapped)?\(", window):
        flags.append("horizontal-parent")
    if re.search(r"add_enabled_ui", window):
        flags.append("add-enabled-ui")
    if re.search(r"ui\.vertical\(|top_down", window):
        flags.append("column-parent")
    return flags


def scan_call_sites(include_tests: bool) -> list[CallSite]:
    sites: list[CallSite] = []
    for base in (APP_SRC, UI_SRC):
        for path in sorted(base.rglob("*.rs")):
            rel = "/" + str(path.relative_to(ROOT))
            if not include_tests and (any(part in rel for part in SKIP_PARTS) or path.name in SKIP_NAMES):
                continue
            text = read(path)
            for m in SELECT_RE.finditer(text):
                expr = statement_from(text, m.start())
                if ".show(" not in expr:
                    continue
                site = CallSite(path=path, line=line_of(text, m.start()), salt=salt_of(expr))
                site.theme = ".theme(" in expr
                site.label = ".label(" in expr
                size = call_arg(expr, "size")
                if size:
                    site.size = "sm" if "Sm" in size else "default" if "Default" in size else size
                variant = call_arg(expr, "variant")
                if variant:
                    site.variant = "ghost" if "Ghost" in variant else "outline" if "Outline" in variant else variant
                width = call_arg(expr, "width")
                if width is not None:
                    site.width = width
                    if re.fullmatch(r"-?[0-9_.]+(?:f32)?", width):
                        site.flags.append("width-literal")
                if not site.theme:
                    site.flags.append("missing-theme")
                before = text[max(0, m.start() - 800) : m.start()]
                if re.search(r"\.small\(\)", before):
                    site.hand_rolled_label = True
                    site.flags.append("hand-rolled-label?")
                site.contexts = context_flags(text, m.start())
                sites.append(site)
    return sites


def component_contract() -> list[dict]:
    findings: list[dict] = []
    for path in sorted(COMPONENTS.rglob("*.rs")):
        text = read(path)
        rel = str(path.relative_to(ROOT))
        for i, line in enumerate(text.splitlines(), start=1):
            if THEME_DEFAULT_RE.search(line):
                findings.append(
                    {
                        "rule": "THEME-NO-LIGHT-DEFAULT",
                        "file": f"{rel}:{i}",
                        "detail": "component builder defaults to the light theme",
                    }
                )
    select_show = COMPONENTS / "select" / "ui" / "select.rs"
    if select_show.exists():
        text = read(select_show)
        for i, line in enumerate(text.splitlines(), start=1):
            if "with_layout(egui::Layout::top_down" in line:
                findings.append(
                    {
                        "rule": "LAYOUT-NEUTRAL",
                        "file": f"{select_show.relative_to(ROOT)}:{i}",
                        "detail": "component show() creates a top_down child UI that re-anchors the control",
                    }
                )
    option = COMPONENTS / "select" / "ui" / "option.rs"
    if option.exists():
        text = read(option)
        if "widget_info" not in text:
            findings.append(
                {
                    "rule": "A11Y-OPTIONS",
                    "file": str(option.relative_to(ROOT)),
                    "detail": "painted option rows expose no widget_info",
                }
            )
        if "allocate_exact_size" in text and "TABLE_ROW_HEIGHT_COMPACT" in text and "painter().layout" in text:
            findings.append(
                {
                    "rule": "MENU-ROW-FIT",
                    "file": str(option.relative_to(ROOT)),
                    "detail": "fixed-height row with wrapping painted text",
                }
            )
    return findings


def capture_hook_findings(sites: list[CallSite]) -> list[dict]:
    if not CAPTURE_RS.exists():
        return []
    text = read(CAPTURE_RS)
    findings = []
    for m in POPUP_OPEN_RE.finditer(text):
        popup_id = m.group(1)
        matching = [s for s in sites if s.salt == popup_id]
        if matching:
            findings.append(
                {
                    "rule": "CAPTURE-DRIVABLE",
                    "file": f"{CAPTURE_RS.relative_to(ROOT)}:{line_of(text, m.start())}",
                    "detail": f"Popup::open_id('{popup_id}') cannot open a Select that stores state in temp data",
                }
            )
    return findings


def render_md(sites: list[CallSite], contract: list[dict]) -> str:
    lines = ["# UI consistency scan", "", f"Call sites: {len(sites)}", ""]
    lines.append("| file:line | salt | size | variant | width | theme | label | context | flags |")
    lines.append("|---|---|---|---|---|---|---|---|---|")
    for s in sites:
        rel = s.path.relative_to(ROOT)
        lines.append(
            "| {}:{} | `{}` | {} | {} | {} | {} | {} | {} | {} |".format(
                rel,
                s.line,
                s.salt,
                s.size,
                s.variant,
                s.width,
                "yes" if s.theme else "NO",
                "yes" if s.label else "no",
                ",".join(s.contexts) or "-",
                ", ".join(s.flags) or "-",
            )
        )
    lines.append("")
    lines.append("## Contract findings")
    lines.append("")
    if contract:
        lines.append("| rule | file | detail |")
        lines.append("|---|---|---|")
        for c in contract:
            lines.append(f"| {c['rule']} | {c['file']} | {c['detail']} |")
    else:
        lines.append("None.")
    lines.append("")
    counts: dict[str, int] = {}
    for s in sites:
        for f in s.flags:
            counts[f] = counts.get(f, 0) + 1
    lines.append("## Flag counts")
    lines.append("")
    for key in sorted(counts):
        lines.append(f"- {key}: {counts[key]}")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("md", "json"), default="md")
    parser.add_argument("--include-tests", action="store_true")
    parser.add_argument("--strict", action="store_true", help="exit 1 on contract violations")
    args = parser.parse_args()

    sites = scan_call_sites(args.include_tests)
    contract = component_contract() + capture_hook_findings(sites)

    if args.format == "json":
        payload = {
            "call_sites": [
                {
                    "file": str(s.path.relative_to(ROOT)),
                    "line": s.line,
                    "salt": s.salt,
                    "size": s.size,
                    "variant": s.variant,
                    "width": s.width,
                    "theme": s.theme,
                    "label": s.label,
                    "contexts": s.contexts,
                    "flags": s.flags,
                }
                for s in sites
            ],
            "contract": contract,
        }
        print(json.dumps(payload, indent=2))
    else:
        print(render_md(sites, contract))

    if args.strict and contract:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
