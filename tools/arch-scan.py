#!/usr/bin/env python3
"""arch-scan — does the code respect the architecture the repo claims to have?

`cc-scan.py` measures the *inside* of a module: functions, naming, comments, error handling.
This tool measures the *outside*: who may import whom. Clean code inside a tangled dependency
graph still rots — you cannot test the domain because it drags in a database driver, and one
handler edit ripples through eleven features.

Heuristic over import statements only: no type checker, no build graph, no module resolution.
Cheap enough to run on every commit. Rules:

  UPWARD_DEPENDENCY        an inner layer imports an outer one            (the dependency rule)
  LAYER_CYCLE              layers form a ring                             (nothing is testable alone)
  DOMAIN_FRAMEWORK_IMPORT  a driver/framework import inside the domain    (ports & adapters)
  BOUNDARY_LEAK            one feature reaches into another's internals
  UNCLASSIFIED_FILES       the declared layering no longer describes the repo
  LAYER_UNUSED             a declared layer has no files

Usage:
    python3 arch-scan.py .                            # text report
    python3 arch-scan.py src --json -o arch.json
    python3 arch-scan.py . --fail-on error            # CI gate
    python3 arch-scan.py --list-rules
    python3 arch-scan.py --explain UPWARD_DEPENDENCY

Stdlib only. Describe your own layers in `arch-scan.config.json` (or `architecture.config.json`)
inside the directory you scan; the defaults cover the classic shape
(domain / application / infrastructure / interface).
"""
from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import signal
import sys
from collections import defaultdict
from pathlib import Path

VERSION = "1.0.0"

CODE_SUFFIXES = {".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".py", ".java",
                 ".kt", ".go", ".cs", ".rs", ".scala"}

# rank 0 = innermost. A file may import layers of rank <= its own rank, never outward.
DEFAULTS: dict = {
    "layers": [
        {"name": "domain", "rank": 0,
         "include": ["**/domain/**", "**/models/**", "**/entities/**", "**/core/**"]},
        {"name": "application", "rank": 1,
         "include": ["**/application/**", "**/use_cases/**", "**/usecase/**",
                     "**/ports/**", "**/services/**", "**/logic/**"]},
        {"name": "infrastructure", "rank": 2,
         "include": ["**/infrastructure/**", "**/persistence/**", "**/repository/**",
                     "**/adapters/**", "**/gateways/**", "**/db/**"]},
        {"name": "interface", "rank": 3,
         "include": ["**/interface/**", "**/interfaces/**", "**/entrypoints/**", "**/api/**",
                     "**/controllers/**", "**/handlers/**",
                     "**/routes/**", "**/views/**", "**/cli/**", "main.py", "main.go"]},
    ],
    # Packages that must never be imported directly by these layers.
    "frameworkPackages": {
        "domain": ["react", "vue", "@angular", "svelte", "express", "fastify", "koa",
                   "sqlalchemy", "django", "flask", "fastapi", "pymysql", "psycopg2",
                   "org.springframework", "org.hibernate", "javax.persistence",
                   "jakarta.persistence", "net/http", "gorm.io", "database/sql"],
        "application": ["sqlalchemy", "pymysql", "psycopg2", "org.hibernate",
                        "javax.servlet", "jakarta.servlet", "express", "react"],
    },
    # Vertical slices: a feature may be reached only through its public API.
    "boundaries": [
        {"name": "features", "root": "features",
         "api": ["**/index.ts", "**/index.tsx", "**/api.ts", "**/contracts.ts",
                 "**/api/__init__.py", "**/public/**"]}
    ],
    "rootPackages": [],
    "testPathPatterns": [r"(^|/)(tests?|__tests__|spec)(/|$)", r"\.(test|spec)\.[jt]sx?$",
                         r"(^|/)test_[^/]*\.py$", r"_test\.go$", r"(Test|Tests)\.(java|kt)$"],
    "ignoreDirs": ["dist", "build", "out", "target", "vendor", "coverage", "generated",
                   "gen", "migrations", "site-packages"],
    "ignoreGlobs": ["*.min.js", "*.min.css", "*.generated.*", "*.d.ts"],
    "maxUnclassifiedPct": 15.0,
}

RULES: dict = {
    "UPWARD_DEPENDENCY": ("inner layer importing an outer one — invert with a port", "error"),
    "LAYER_CYCLE": ("two or more layers depend on each other in a ring", "error"),
    "DOMAIN_FRAMEWORK_IMPORT": ("framework or driver package inside a layer that must stay pure",
                                "error"),
    "BOUNDARY_LEAK": ("one feature importing another feature's internals", "error"),
    "UNCLASSIFIED_FILES": ("too many code files match no declared layer", "info"),
    "LAYER_UNUSED": ("a declared layer matches no file at all", "info"),
}

# One hint per rule: the sentence a human acts on. It never varied per call site, so it lives
# here instead of being retyped at each `_finding(...)` - that is what kept the constructor at
# six parameters (HARD_PARAMS on the tool's own scan).
HINTS = {
    "UPWARD_DEPENDENCY":
        "Point the dependency inward: own the abstraction here, let the outer layer implement it.",
    "LAYER_CYCLE":
        "Break the ring with a port or an event — inside a cycle no layer can be tested or "
        "replaced alone.",
    "DOMAIN_FRAMEWORK_IMPORT":
        "Declare a port (interface) in this layer, implement it in infrastructure, wire it at "
        "the composition root.",
    "BOUNDARY_LEAK":
        "Import the feature's public API, publish an event, or move the shared piece into a "
        "kernel.",
    "UNCLASSIFIED_FILES":
        "Add their folders to a layer in arch-scan.config.json, or admit the code has no "
        "architecture yet.",
    "LAYER_UNUSED":
        "Delete the layer from the config or create it in the repo — a diagram nobody implements "
        "is worse than no diagram.",
}

EXPLAIN = {
    "UPWARD_DEPENDENCY":
        "The dependency rule: source code dependencies point inwards, towards the domain. A "
        "domain module importing an HTTP framework, a repository importing a controller, a use "
        "case importing a view — all the same mistake in different clothes. Fix it by declaring "
        "an interface in the inner layer and letting the outer layer implement it.",
    "LAYER_CYCLE":
        "Two layers that import each other cannot be tested, replaced or extracted alone, so the "
        "boundary is decoration. Break the ring: invert one edge with a port, or publish an event "
        "instead of calling back.",
    "DOMAIN_FRAMEWORK_IMPORT":
        "Frameworks optimise for their own lifecycle, not your business rules. Once `sqlalchemy` "
        "or `express` types appear in the domain, your business rules cannot be tested without "
        "that framework. Keep the domain in plain types; map at the edge.",
    "BOUNDARY_LEAK":
        "Features (bounded contexts) talk through a published contract: an API module, an event, "
        "a shared kernel. Reaching into another feature's internals turns their refactor into "
        "your incident.",
    "UNCLASSIFIED_FILES":
        "A layer diagram that most of the repo ignores is a poster, not a constraint. Either "
        "describe reality in arch-scan.config.json or move the files.",
    "LAYER_UNUSED":
        "A declared layer with no files is drift: the folder was never created, or was renamed. "
        "Fix the config or build the layer.",
}

ALWAYS_IGNORE_DIRS = frozenset({".git", ".hg", ".svn", "node_modules", "__pycache__",
                                ".mypy_cache", ".ruff_cache", ".pytest_cache", ".venv", "venv"})
ALLOW_RE = re.compile(r"(?://|#|/\*)\s*arch-scan:allow\s+([A-Za-z_,\s]+)")
GO_BLOCK_ITEM = re.compile(r'^(?:[\w.]+\s+)?"([^"]+)"')
IMPORT_PATTERNS: dict[str, tuple[re.Pattern, ...]] = {
    "py": (re.compile(r"^\s*from\s+([.\w]+)\s+import\b"),
           re.compile(r"^\s*import\s+([.\w]+)")),
    "ts": (re.compile("""(?:^|[^\\w.])(?:from|import)\\s*["']([^"']+)["']"""),
           re.compile("""require\\(\\s*["']([^"']+)["']"""),
           re.compile("""import\\(\\s*["']([^"']+)["']""")),
    "jvm": (re.compile(r"^\s*import\s+(?:static\s+)?([A-Za-z_][\w.]*)\s*;"),),
    "go": (re.compile(r'^\s*import\s+(?:[\w.]+\s+)?"([^"]+)"'),),
}


def lang_of(path: Path) -> str | None:
    ext = path.suffix
    if ext in {".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"}:
        return "ts"
    if ext == ".py":
        return "py"
    if ext in {".java", ".kt", ".scala"}:
        return "jvm"
    if ext == ".go":
        return "go"
    return None


def package_name(spec: str) -> str:
    """Package identity: '@scope/pkg' for scoped names, first segment otherwise."""
    if spec.startswith("@"):
        return "/".join(spec.split("/")[:2])
    return spec.split("/")[0]


def extract_imports(path: Path, text: str) -> list[tuple[int, str]]:
    """(line, specifier) for every import we can see. Deliberately simple: regex, no parser."""
    lang = lang_of(path)
    if not lang:
        return []
    patterns = IMPORT_PATTERNS[lang]
    out: list[tuple[int, str]] = []
    in_go_block = False
    for line_no, raw in enumerate(text.split("\n"), 1):
        code = raw.strip()
        if lang == "go":
            if in_go_block:
                in_go_block = not code.startswith(")")
                if not code.startswith(")") and not code.startswith("//"):
                    m = GO_BLOCK_ITEM.match(code)
                    if m:
                        out.append((line_no, m.group(1)))
                continue
            if code.startswith("import ("):
                in_go_block = True
                continue
        if code.startswith(("//", "#", "*", "/*")):
            continue
        for rx in patterns:
            m = rx.search(raw)
            if m:
                spec = m.group(1).strip()
                if spec and not spec.endswith((".css", ".svg", ".png", ".json", ".scss")):
                    out.append((line_no, spec))
                break
    return out


def path_match(rel: str, patterns: list) -> bool:
    """Probe a few shapes: an import specifier names a *directory* (`internal/adapters`) with no
    trailing segment, while a file path has one. `**/domain/**` must match both."""
    probes = (rel, "/" + rel, rel + "/", "/" + rel + "/")
    for pattern in patterns:
        variants = [pattern]
        if pattern.startswith("**/"):
            variants.append(pattern[3:])
        for probe in probes:
            if any(fnmatch.fnmatch(probe, variant) for variant in variants):
                return True
    return False


class Layout:
    """Maps file paths and import specifiers onto declared layers and boundaries."""

    def __init__(self, cfg: dict, root: Path):
        self.root = root
        self.layers = sorted(cfg["layers"], key=lambda item: int(item["rank"]))
        self.ranks = {str(item["name"]): int(item["rank"]) for item in self.layers}
        self.framework = {str(k): [str(x) for x in v]
                          for k, v in (cfg.get("frameworkPackages") or {}).items()}
        self.boundaries = list(cfg.get("boundaries") or [])
        self.root_packages = [str(p) for p in (cfg.get("rootPackages") or [])]

    def layer_of(self, rel: str) -> str | None:
        for layer in self.layers:
            if path_match(rel, list(layer.get("include", []))):
                return str(layer["name"])
        return None

    def boundary_of(self, rel: str) -> tuple[str, str] | None:
        """`src/features/checkout/…` -> ('features', 'checkout'); needs at least one inner file."""
        parts = rel.split("/")
        for bound in self.boundaries:
            marker = str(bound.get("root", ""))
            if marker and marker in parts[:-2]:
                idx = parts.index(marker)
                return str(bound.get("name", marker)), parts[idx + 1]
        return None

    def is_public_api(self, boundary: str, rel: str) -> bool:
        for bound in self.boundaries:
            if str(bound.get("name", "")) == boundary:
                return path_match(rel, list(bound.get("api", [])))
        return False

    def resolve(self, src_rel: str, spec: str, lang: str = "ts") -> str | None:
        """Relative specifier -> repo-relative file, or None when we cannot tell.

        Python uses dots for both "go up a package" and "path separator" (`..db.pool`),
        JS/TS use `../db/pool`, Java/Go never come here (they are absolute and matched
        against `rootPackages` instead).
        """
        if not spec.startswith("."):
            return None
        base = (self.root / src_rel).parent
        if lang == "py":
            dots = len(spec) - len(spec.lstrip("."))
            remainder = spec[dots:].replace(".", "/").strip("/")
            for _ in range(dots - 1):
                base = base.parent
            target = (base / remainder).resolve() if remainder else base.resolve()
        else:
            target = (base / spec).resolve()
        for cand in (target, target.with_suffix(""), target / "__init__.py", target / "index.ts",
                     target / "index.tsx", target / "index.js", Path(f"{target}.ts"),
                     Path(f"{target}.tsx"), Path(f"{target}.py")):
            try:
                if cand.is_file():
                    return cand.resolve().relative_to(self.root).as_posix()
            except (OSError, ValueError):
                continue
        return None

    def layer_of_spec(self, spec: str) -> str | None:
        """Layer for a dotted/slash import (Java, Go, Python absolute) when it is internal.

        Two routes: the declared `rootPackages` prefix, or — when nobody configured that yet —
        "the dotted name happens to be a real folder inside this repo". The fallback keeps a
        fresh copy of this tool from silently passing a project whose imports start with `app/`.
        """
        if not self.root_packages:
            guess = spec.replace(".", "/").replace("__", "_")
            if guess and (self.root / f"{guess}.py").exists() or (self.root / guess).is_dir():
                return self.layer_of(f"{guess}" if (self.root / guess).is_dir() else f"{guess}.py")
        for prefix in self.root_packages:
            for form in (prefix, prefix.replace(".", "/")):
                if spec.startswith(form + ".") or spec.startswith(form + "/"):
                    remainder = spec[len(form):].lstrip("./").replace(".", "/")
                    return self.layer_of(remainder)
        return None

    def framework_in(self, layer: str | None, spec: str) -> str | None:
        """The banned package this import belongs to, if this layer may not touch it."""
        if layer is None or layer not in self.framework:
            return None
        pkg = package_name(spec)
        for banned in self.framework[layer]:
            if spec == banned or spec.startswith(f"{banned}.") or spec.startswith(f"{banned}/") \
                    or pkg == banned:
                return banned
        return None


def iter_files(root: Path, cfg: dict) -> list[Path]:
    ignore = ALWAYS_IGNORE_DIRS | {str(d) for d in (cfg.get("ignoreDirs") or [])}
    globs = [str(g) for g in (cfg.get("ignoreGlobs") or [])]
    out: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(d for d in dirnames if d not in ignore and not d.startswith("."))
        for name in sorted(filenames):
            path = Path(dirpath) / name
            if path.suffix not in CODE_SUFFIXES:
                continue
            if any(fnmatch.fnmatch(name, g) for g in globs):
                continue
            try:
                if path.stat().st_size > 1_000_000:
                    continue
            except OSError:
                continue
            out.append(path)
    return out


def is_test(rel: str, cfg: dict) -> bool:
    return any(re.search(str(p), rel) for p in cfg.get("testPathPatterns") or [])


def allow_map(text: str) -> dict[int, set[str]]:
    """`# arch-scan:allow RULE` covers its own line and the line below it."""
    allowed: dict[int, set[str]] = {}
    for idx, line in enumerate(text.split("\n"), 1):
        m = ALLOW_RE.search(line)
        if m:
            rules = {x.strip().upper() for x in m.group(1).split(",") if x.strip()}
            allowed.setdefault(idx, set()).update(rules)
            allowed.setdefault(idx + 1, set()).update(rules)
    return allowed


def _finding(rule: str, rel: str, line: int, message: str, value) -> dict:
    """One violation. `value` is the machine-facing key (pair, package, ring); the human-facing
    `hint` comes from HINTS, because it is a property of the rule, not of the call site."""
    return {"rule": rule, "severity": RULES[rule][1], "file": rel, "line": line,
            "message": message, "value": value, "hint": HINTS[rule]}


def find_cycles(graph: dict[str, set[str]]) -> list[list[str]]:
    """Tarjan strongly-connected components; recursive because layers number in single digits."""
    index: dict[str, int] = {}
    low: dict[str, int] = {}
    stack: list[str] = []
    on_stack: set[str] = set()
    rings: list[list[str]] = []

    def strong(node: str) -> None:
        index[node] = low[node] = len(index)
        stack.append(node)
        on_stack.add(node)
        for nxt in sorted(graph.get(node, set())):
            if nxt not in index:
                strong(nxt)
                low[node] = min(low[node], low[nxt])
            elif nxt in on_stack:
                low[node] = min(low[node], index[nxt])
        if low[node] == index[node]:
            ring = []
            while True:
                last = stack.pop()
                on_stack.discard(last)
                ring.append(last)
                if last == node:
                    break
            if len(ring) > 1:
                rings.append(sorted(ring))

    for node in sorted(graph):
        if node not in index:
            strong(node)
    return rings


def scan_tree(root: Path, cfg: dict, files: list[Path]) -> dict:
    layout = Layout(cfg, root)
    edges: dict[tuple[str, str], list[str]] = defaultdict(list)
    graph: dict[str, set[str]] = {name: set() for name in layout.ranks}
    findings: list[dict] = []
    hits: dict[str, int] = {name: 0 for name in layout.ranks}
    unclassified: list[str] = []
    seen_leaks: set[tuple[str, str]] = set()
    slices: dict[str, int] = defaultdict(int)

    for path in files:
        try:
            rel = path.relative_to(root).as_posix()
        except ValueError:
            rel = path.name
        try:
            text = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        src_layer = layout.layer_of(rel)
        src_boundary = layout.boundary_of(rel)
        if src_layer:
            hits[src_layer] += 1
        elif src_boundary:
            slices[f"{src_boundary[0]}/{src_boundary[1]}"] += 1
        elif not is_test(rel, cfg):
            unclassified.append(rel)
        allowed = allow_map(text)
        lang = lang_of(path) or "ts"

        for line_no, spec in extract_imports(path, text):
            skip = allowed.get(line_no, set())
            if "ALL" in skip:
                continue
            target = layout.resolve(rel, spec, lang)
            if target is None and spec.startswith("."):
                continue  # relative import we cannot place — out of scope for a heuristic
            tgt_layer = layout.layer_of(target) if target else layout.layer_of_spec(spec)

            if tgt_layer is None and not spec.startswith("."):
                if src_layer and "DOMAIN_FRAMEWORK_IMPORT" not in skip:
                    banned = layout.framework_in(src_layer, spec)
                    if banned:
                        findings.append(_finding(
                            "DOMAIN_FRAMEWORK_IMPORT", rel, line_no,
                            f"`{spec}` is a driver/framework import in the '{src_layer}' layer",
                            f"{src_layer}:{banned}"))
                continue
            tgt_boundary = layout.boundary_of(target) if target else None
            if src_boundary and tgt_boundary and src_boundary[0] == tgt_boundary[0] \
                    and src_boundary[1] != tgt_boundary[1] and target:
                pair = (src_boundary[1], tgt_boundary[1])
                public = layout.is_public_api(src_boundary[0], target)
                if not public and pair not in seen_leaks and "BOUNDARY_LEAK" not in skip:
                    seen_leaks.add(pair)
                    findings.append(_finding(
                        "BOUNDARY_LEAK", rel, line_no,
                        f"feature '{src_boundary[1]}' imports '{tgt_boundary[1]}' internals "
                        f"({target})", f"{pair[0]}->{pair[1]}"))

            if not src_layer or not tgt_layer or tgt_layer == src_layer:
                continue
            edges[(src_layer, tgt_layer)].append(rel)
            graph[src_layer].add(tgt_layer)
            if layout.ranks[tgt_layer] > layout.ranks[src_layer] \
                    and "UPWARD_DEPENDENCY" not in skip:
                findings.append(_finding(
                    "UPWARD_DEPENDENCY", rel, line_no,
                    f"'{src_layer}' imports outward into '{tgt_layer}' via `{spec}`",
                    f"{src_layer}->{tgt_layer}"))

    findings.extend(_cycle_findings(graph, edges))
    findings.extend(_drift_findings(cfg, files, hits, unclassified))
    return {"files": len(files), "findings": findings, "edges": dict(edges), "hits": hits,
            "slices": dict(slices), "unclassified": unclassified}


def _cycle_findings(graph: dict[str, set[str]], edges: dict) -> list[dict]:
    out = []
    for ring in find_cycles(graph):
        arrows = ", ".join(f"{a}->{b}" for a, b in zip(ring, ring[1:] + ring[:1]))
        where = next((f for layer in ring for f in edges.get((layer, ring[0]), [])), "(project)")
        out.append(_finding("LAYER_CYCLE", where, 0,
                      f"Layers depend on each other in a ring: {arrows}", "+".join(ring)))
    return out


def _drift_findings(cfg: dict, files: list[Path], hits: dict[str, int],
                    unclassified: list[str]) -> list[dict]:
    out: list[dict] = []
    total = len(files)
    pct = round(100.0 * len(unclassified) / total, 1) if total else 0.0
    limit = float(cfg.get("maxUnclassifiedPct", 15.0))
    if total and pct > limit:
        out.append(_finding("UNCLASSIFIED_FILES", "(project)", 0,
                      f"{len(unclassified)}/{total} code files ({pct}%) match no declared layer "
                      f"(threshold {limit}%)", pct))
    for name, count in sorted(hits.items()):
        if count == 0:
            out.append(_finding("LAYER_UNUSED", "(config)", 0,
                          f"Declared layer '{name}' matches no file", name))
    return out


def load_config(explicit: str | None, root: Path) -> dict:
    cfg = json.loads(json.dumps(DEFAULTS))  # deep copy: callers must not mutate the defaults
    candidates = [Path(explicit)] if explicit else [root / "arch-scan.config.json",
                                                     root / "architecture.config.json"]
    for cand in candidates:
        if not cand.is_file():
            continue
        try:
            loaded = json.loads(cand.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            print(f"[arch-scan] {cand} is not valid JSON: {exc}", file=sys.stderr)
            break
        for key in ("layers", "boundaries", "frameworkPackages", "rootPackages",
                    "ignoreGlobs", "testPathPatterns"):
            if isinstance(loaded.get(key), (list, dict)) and loaded[key]:
                cfg[key] = loaded[key]
        if isinstance(loaded.get("ignoreDirs"), list):
            cfg["ignoreDirs"] = sorted(set(DEFAULTS["ignoreDirs"]) | set(loaded["ignoreDirs"]))
        if "maxUnclassifiedPct" in loaded:
            cfg["maxUnclassifiedPct"] = loaded["maxUnclassifiedPct"]
        cfg["_configPath"] = str(cand)
        break
    return cfg


def grade(score: float) -> str:
    return "A" if score >= 90 else "B" if score >= 80 else "C" if score >= 70 else \
        "D" if score >= 55 else "E"


def build_report(root: Path, cfg: dict, scanned: dict) -> dict:
    findings = sorted(scanned["findings"], key=lambda f: (f["file"], f["line"], f["rule"]))
    counts = {"error": 0, "warning": 0, "info": 0}
    for item in findings:
        counts[item["severity"]] = counts.get(item["severity"], 0) + 1
    score = max(0.0, round(100.0 - counts["error"] * 6 - counts["info"], 1))
    return {"tool": "arch-scan", "version": VERSION, "root": str(root),
            "config": {"_configPath": cfg.get("_configPath"),
                       "layers": [f"{layer['name']} (rank {layer['rank']})"
                                  for layer in cfg["layers"]],
                       "rootPackages": cfg.get("rootPackages"),
                       "maxUnclassifiedPct": cfg.get("maxUnclassifiedPct")},
            "filesScanned": scanned["files"], "score": score, "grade": grade(score),
            "counts": counts, "layers": scanned["hits"], "slices": scanned.get("slices", {}),
            "edges": {f"{a}->{b}": len(v) for (a, b), v in scanned["edges"].items()},
            "findings": findings}


def print_report(rep: dict) -> None:
    counts = rep["counts"]
    print(f"arch-scan v{rep['version']} · {rep['filesScanned']} file · {rep['root']}")
    print(f"Architecture score: {rep['score']}/100 (grade {rep['grade']})  ·  "
          f"error={counts['error']} info={counts['info']}")
    print("\nLayer census:")
    for name, count in sorted(rep["layers"].items(), key=lambda kv: (-kv[1], kv[0])):
        print(f"  {name:<16} {count:>4}")
    if rep.get("slices"):
        print("Vertical slices:")
        for name, count in sorted(rep["slices"].items()):
            print(f"  {name:<16} {count:>4}")
    if rep["edges"]:
        banned = {str(item["value"]) for item in rep["findings"]
                  if item["rule"] == "UPWARD_DEPENDENCY" and item["value"]}
        print("\nLayer dependencies:")
        for label, count in sorted(rep["edges"].items(), key=lambda kv: (-kv[1], kv[0])):
            src, tgt = label.split("->", 1)
            mark = "x" if label in banned else " "
            print(f"  {mark} {src:<16} -> {tgt:<16} {count:>3} import(s)")
    if not rep["findings"]:
        print("\nOK — every import we can see points in the allowed direction.")
        return
    print("\nFindings:")
    for item in rep["findings"]:
        loc = f"{item['file']}:{item['line']}" if item["line"] else item["file"]
        print(f"  {loc:<46} {item['severity']:<5} {item['rule']}")
        print(f"  {'':<46} {item['message']}")
        if item["hint"]:
            print(f"  {'':<46} -> {item['hint']}")
    print("\nCI suggestion: python3 arch-scan.py . --fail-on error")


def collect_targets(targets: list[Path], cfg: dict) -> tuple[Path, list[Path]]:
    """Single directory scans the whole tree; several paths merge, each relative to itself."""
    if len(targets) == 1:
        only = targets[0]
        return (only, iter_files(only, cfg)) if only.is_dir() else (only.parent, [only])
    root = Path.cwd()
    files: list[Path] = []
    for one in targets:
        files.extend(iter_files(one, cfg) if one.is_dir() else [one])
    return root, sorted(set(files))


def main(argv: list[str] | None = None) -> int:
    if hasattr(signal, "SIGPIPE"):
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)  # `| head` stays quiet, like grep
    ap = argparse.ArgumentParser(prog="arch-scan",
                                 description="Check import direction against declared layers.")
    ap.add_argument("paths", nargs="*", default=["."], help="file or directory to scan")
    ap.add_argument("--config", help="path to arch-scan.config.json")
    ap.add_argument("--json", action="store_true", help="machine-readable report")
    ap.add_argument("-o", "--output", help="write the JSON report to a file")
    ap.add_argument("--fail-on", choices=["none", "info", "error"], default="error",
                    help="exit 1 on findings of this severity (default: error)")
    ap.add_argument("--list-rules", action="store_true", help="print the rule catalogue")
    ap.add_argument("--explain", metavar="RULE", help="print one rule in detail")
    ap.add_argument("--version", action="store_true", help="print version")
    args = ap.parse_args(argv)

    if args.version:
        print(f"arch-scan {VERSION}")
        return 0
    if args.list_rules:
        for name, (description, severity) in RULES.items():
            print(f"{name:<26} {severity:<6} {description}")
        return 0
    if args.explain:
        name = args.explain.upper()
        if name not in RULES:
            print(f"[arch-scan] unknown rule: {name}", file=sys.stderr)
            return 2
        print(f"{name} ({RULES[name][1]}) — {RULES[name][0]}\n\n{EXPLAIN[name]}")
        return 0

    targets = [Path(p).resolve() for p in (args.paths or ["."])]
    missing = [str(t) for t in targets if not t.exists()]
    for name in missing:
        print(f"[arch-scan] not found: {name}", file=sys.stderr)
    targets = [t for t in targets if t.exists()]
    if not targets:
        return 2

    root = targets[0] if len(targets) == 1 and targets[0].is_dir() else Path.cwd()
    cfg = load_config(args.config, root)
    root, files = collect_targets(targets, cfg)
    scanned = scan_tree(root, cfg, files)
    rep = build_report(root, cfg, scanned)

    out = json.dumps(rep, indent=2, ensure_ascii=False)
    if args.output:
        Path(args.output).write_text(out + "\n", encoding="utf-8")
        print(f"[arch-scan] wrote report: {args.output}")
    if args.json:
        print(out)
    elif not args.output:
        print_report(rep)

    if args.fail_on == "none" or rep["score"] == 100.0:
        return 0
    return 1 if rep["counts"]["error"] or (args.fail_on == "info" and rep["counts"]["info"]) else 0


if __name__ == "__main__":
    raise SystemExit(main())
