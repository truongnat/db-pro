#!/usr/bin/env python3
"""
cc-scan — Clean Code scanner (heuristic, stdlib-only, zero dependencies).

Scan a repository and report the violations standardised by the `clean-code` skill pack.
(xem ../skills/clean-code/SKILL.md):

  * functions too long / too complex / too many parameters / boolean flag arguments
  * magic numbers, over-wide lines, mixed indentation, trailing whitespace
  * commented-out code, leftover TODO/FIXME, error-swallowing catch blocks
  * leftover debug logging (console.log / System.out / print / fmt.Print)
  * deep nesting, duplicated blocks (DRY), low ratio of test files

Designed for CI: exit code 1 when a finding at or above --fail-on exists.

Quick start:
    python3 cc-scan.py .
    python3 cc-scan.py src --json -o cc-report.json
    python3 cc-scan.py . --update-baseline     # freeze the legacy state, then ratchet
    python3 cc-scan.py . --list-rules

Warning: this is a heuristic (regex + bracket counting), NOT a full parser.
Findings are signals for a human reviewer, not verdicts.
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

VERSION = "1.0.1"

# --------------------------------------------------------------------------------------
# Defaults (override with clean-code.config.json / .clean-code.json / CLI flags)
# --------------------------------------------------------------------------------------
DEFAULTS: dict = {
    "maxLineLength": 120,
    "maxFunctionLines": 40,
    "hardFunctionLines": 70,
    "maxParams": 3,
    "hardParams": 5,
    "maxNesting": 3,
    "maxComplexity": 10,
    "hardComplexity": 15,
    "magicNumbersAllowed": ["0", "1", "-1", "2"],
    "dupMinLines": 8,
    "minTestRatio": 0.15,
    "ignoreDirs": [
        ".git", ".hg", ".svn", "node_modules", "bower_components", "vendor",
        "dist", "build", "out", "target", "bin", "obj", "coverage", ".nyc_output",
        ".next", ".nuxt", ".output", ".cache", ".pytest_cache", ".mypy_cache",
        ".ruff_cache", "__pycache__", ".venv", "venv", "env", ".idea", ".vscode",
        "generated", "gen", "migrations", "Pods", "DerivedData", "tmp", "temp",
    ],
    "ignoreGlobs": ["*.min.js", "*.min.css", "*.bundle.js", "*.lock",
                    "package-lock.json", "yarn.lock", "*.g.dart", "*.generated.*"],
    "testPathPatterns": [
        r"(^|/)(tests?|__tests__|spec|specs)(/|$)",
        r"\.(test|spec)\.[jt]sx?$",
        r"(^|/)test_[^/]*\.py$",
        r"_test\.go$",
        r"(Test|Tests|IT)\.(java|kt)$",
    ],
    "skipRules": [],
    "skipRulesForTests": ["MAGIC_NUMBER", "LONG_FUNCTION", "HUGE_FUNCTION", "DEBUG_STATEMENT",
                          "TOO_MANY_PARAMS", "LINE_TOO_LONG", "COMPLEXITY"],
}

SEVERITY_ORDER = {"error": 3, "warning": 2, "info": 1}

# Minimum protection floor - no config file may remove it. Dependency and cache folders
# are never what you want to lint. The filter applies to child folders only: to really scan
# one, pass it as the path argument (then the root itself is not filtered).
ALWAYS_IGNORE_DIRS = frozenset({
    ".git", ".hg", ".svn", "node_modules", "bower_components", "package-lock.json",
    "__pycache__", ".mypy_cache", ".ruff_cache", ".pytest_cache", ".venv", "venv",
})


RULES: dict = {
    "LONG_FUNCTION":       ("Function exceeds the line budget", "warning"),
    "HUGE_FUNCTION":       ("Function is huge - split it now", "error"),
    "TOO_MANY_PARAMS":     ("Too many parameters - group them into an options object", "warning"),
    "HARD_PARAMS":         ("Parameter count out of control", "error"),
    "BOOLEAN_PARAM":       ("Boolean flag argument - the function does two things", "info"),
    "COMPLEXITY":          ("High branch complexity", "warning"),
    "HARD_COMPLEXITY":     ("Too many branches to test exhaustively", "error"),
    "DEEP_NESTING":        ("Deeply nested control flow - use guard clauses", "warning"),
    "MAGIC_NUMBER":        ("Magic number - give the constant a name", "warning"),
    "LINE_TOO_LONG":       ("Line exceeds the width limit", "warning"),
    "MIXED_INDENT":        ("Mixed tabs and spaces", "warning"),
    "TRAILING_WHITESPACE": ("Trailing whitespace at end of line", "info"),
    "COMMENTED_CODE":      ("Commented-out code - let Git keep history instead", "warning"),
    "BLOCK_COMMENT":       ("Long comment block - consider moving it to docs", "info"),
    "TODO_MARK":           ("Leftover TODO/FIXME/XXX/HACK", "info"),
    "EMPTY_CATCH":         ("Empty catch swallows the error", "error"),
    "DEBUG_STATEMENT":     ("Debug logging left in source", "warning"),
    "NEGATIVE_CONDITIONAL": ("Hard-to-read double negation", "info"),
    "DUPLICATE_BLOCK":     ("Duplicated code block (violates DRY)", "warning"),
    "LOW_TEST_RATIO":      ("Low ratio of test files to source files", "info"),
}

# --------------------------------------------------------------------------------------
# Languages & splitting code / comment / string
# --------------------------------------------------------------------------------------
# Rules reported at the function declaration line (vs per-statement rules)
FUNCTION_RULES = {"LONG_FUNCTION", "HUGE_FUNCTION", "TOO_MANY_PARAMS", "HARD_PARAMS",
                  "COMPLEXITY", "HARD_COMPLEXITY", "DEEP_NESTING", "BOOLEAN_PARAM"}

PYTHONISH = {"python", "ruby"}
BRACE_LANGS = {"js", "ts", "java", "kotlin", "scala", "groovy", "c", "cpp", "csharp",
               "go", "php", "swift", "rust", "objc", "dart"}
LINE_COMMENT = {**{lang: "//" for lang in BRACE_LANGS}, "python": "#", "ruby": "#"}

EXT_LANG = {
    ".js": "js", ".jsx": "js", ".mjs": "js", ".cjs": "js",
    ".ts": "ts", ".tsx": "ts",
    ".java": "java", ".kt": "kotlin", ".kts": "kotlin", ".scala": "scala",
    ".groovy": "groovy", ".c": "c", ".h": "c", ".cpp": "cpp", ".hpp": "cpp",
    ".cc": "cpp", ".cxx": "cpp", ".cs": "csharp", ".go": "go", ".php": "php",
    ".swift": "swift", ".rs": "rust", ".m": "objc", ".dart": "dart",
    ".py": "python", ".pyw": "python", ".rb": "ruby",
}


def detect_lang(path: Path) -> str | None:
    return EXT_LANG.get(path.suffix.lower())


def split_line(raw: str, lang: str, state: dict) -> tuple[str, str]:
    """Split one line into (code, comment). String and comment bodies become '·'."""
    marker = LINE_COMMENT.get(lang, "//")
    code: list[str] = []
    comment: list[str] = []
    i, n = 0, len(raw)

    while i < n:
        if state.get("openstr"):
            q = state["openstr"]
            j = i
            while j < n:
                if raw[j] == "\\" and lang != "ruby":
                    j += 2
                    continue
                if raw.startswith(q, j):
                    state["openstr"] = None
                    j += len(q)
                    break
                j += 1
            code.append("·" * (min(j, n) - i))
            i = min(j, n)
            continue

        if state.get("in_block"):
            j = raw.find("*/", i)
            if j < 0:
                comment.append(raw[i:])
                i = n
            else:
                comment.append(raw[i:j + 2])
                state["in_block"] = False
                i = j + 2
            continue

        two = raw[i:i + 2]
        if marker == "//" and two == "//":
            comment.append(raw[i + 2:])
            break
        if marker == "#" and raw[i] == "#":
            comment.append(raw[i + 1:])
            break
        if lang in BRACE_LANGS and two == "/*":
            state["in_block"] = True
            comment.append("/*")
            i += 2
            continue
        if raw[i] in "\"'`":
            quote = raw[i]
            if lang in PYTHONISH and two in ('"""', "'''"):
                state["openstr"] = two
                code.append("···")
                i += 3
                continue
            if quote == "`":
                state["openstr"] = "`"
                code.append("·")
                i += 1
                continue
            j, closed = i + 1, False
            while j < n:
                if raw[j] == "\\":
                    j += 2
                    continue
                if raw[j] == quote:
                    j += 1
                    closed = True
                    break
                j += 1
            if closed:
                code.append("·" * (j - i))
                i = j
                continue
            code.append("·" * (n - i))   # bare quote (e.g. in prose) -> treat as plain code
            break
        code.append(raw[i])
        i += 1

    return "".join(code), " ".join("".join(comment).split())


def preprocess(text: str, lang: str) -> tuple[list[str], list[str]]:
    code_lines, comment_lines = [], []
    state = {"in_block": False, "openstr": None}
    for raw in text.split("\n"):
        code, comment = split_line(raw, lang, state)
        code_lines.append(code)
        comment_lines.append(comment)
    return code_lines, comment_lines


def real(line: str) -> str:
    """Drop string/comment placeholders to decide whether a line holds real code."""
    return line.replace("·", "").strip()


# --------------------------------------------------------------------------------------
# Function detection
# --------------------------------------------------------------------------------------
SKIP_HEAD = {
    "if", "for", "while", "switch", "catch", "return", "new", "do", "else", "try",
    "throw", "yield", "await", "delete", "typeof", "in", "of", "case", "synchronized",
    "with", "using", "foreach", "elif", "except", "assert", "print", "super", "when",
    "match", "require", "raise", "lambda", "export", "import", "package", "class",
    "interface", "enum", "record", "struct", "type", "namespace", "module", "declare",
}
CALLBACK_NAMES = {
    "forEach", "map", "filter", "reduce", "then", "catch", "finally", "sort", "some",
    "every", "each", "subscribe", "next", "complete", "useEffect", "useMemo", "test",
    "expect", "describe", "it", "beforeEach", "afterEach", "done", "invoke", "call",
    "apply", "of", "from", "builder", "build", "toString", "equals", "hashCode", "run",
}
MODIFIERS = (r"(?:public|private|protected|internal|static|final|abstract|override|"
             r"readonly|async|suspend|open|sealed|inline|default|virtual|const|"
             r"get|set|operator|companion|strictfp|synchronized|native)")

FN_PREFIX_RE = re.compile(
    r"^\s*(?:(?:export|default)\s+)*(?:(?:async|static|pub|inline|const|virtual|override)\s+)*"
    r"(?:function\s*\*?|func|fn|fun)\s*(?:\([^)]*\)\s*)?([A-Za-z_$]\w*)?\s*\(")

ARROW_RE = re.compile(
    r"^\s*(?:(?:export|export\s+default|const|let|var)\s+)*([A-Za-z_$]\w*)\s*"
    r"(?::[^=;{]+)?=\s*(?:async\s*)?\(([^)]*)\)\s*(?::[^=;{]+)?=>")

DECL_RE = re.compile(
    r"^(?:(?:@[\w.]+(?:\([^)]*\))?\s+|(?:" + MODIFIERS + r")\s+)*"
    r"(?:[\w<>\[\].,?&|*]+\s+)?)"                      # result type
    r"([A-Za-z_$][\w$]*)\s*\(([^;{}]*)\)"              # name + parameters
    r"(?:\s*(?:throws\s+[\w,.\s]+|->\s*[\w<>\[\],.?\s]+|:\s*[\w<>\[\],.?\s]+|async))*"
    r"\s*\{?\s*$")


def _balanced(text: str, open_idx: int) -> tuple[str, bool]:
    depth, out, i = 0, [], open_idx
    while i < len(text):
        c = text[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                return "".join(out[1:]), True
        out.append(c)
        i += 1
    return "".join(out[1:]), False


def extract_params(sig: str, lang: str) -> list[str] | None:
    open_idx = sig.find("(")
    if open_idx < 0:
        return None
    inner, ok = _balanced(sig, open_idx)
    if not ok:
        return None
    parts, depth, cur = [], 0, []
    for c in inner:
        if c in "([{<":
            depth += 1
        elif c in ")]}>":
            depth -= 1
        if c == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(c)
    if "".join(cur).strip():
        parts.append("".join(cur))
    params = [p.strip() for p in parts if p.strip()]
    if lang in PYTHONISH:
        params = [p for p in params if re.split(r"[:=\s*]", p.strip())[0] not in ("self", "cls")]
    return params


def find_functions(code_lines: list[str], lang: str) -> list[dict]:
    """List of functions: name/start/end/params/signature (0-based indexes)."""
    funcs: list[dict] = []
    n = len(code_lines)

    if lang in PYTHONISH:
        py_re = re.compile(r"^(\s*)(?:async\s+)?def\s+([A-Za-z_]\w*)\s*\(")
        for i, line in enumerate(code_lines):
            m = py_re.match(real(line) and line or line)
            if not m:
                continue
            indent = len(m.group(1).expandtabs(4))
            sig, j = line, i
            while sig.count("(") > sig.count(")") and j + 1 < n:
                j += 1
                sig += " " + real(code_lines[j])
            end = i
            for k in range(j + 1, n):
                cur = code_lines[k]
                if not real(cur):
                    continue
                if len(cur[:len(cur) - len(cur.lstrip())].expandtabs(4)) <= indent:
                    break
                end = k
            funcs.append({"name": m.group(2), "start": i, "end": max(end, i),
                          "params": extract_params(sig, lang), "signature": " ".join(sig.split())[:150],
                          "indent": indent})
        return funcs

    i = 0
    while i < n:
        line = code_lines[i]
        s = real(line)
        stripped = line.strip()
        if not s or s.startswith(("@",)):
            i += 1
            continue
        head_token = re.match(r"[A-Za-z_$][\w$]*", stripped)
        if head_token and head_token.group(0) in SKIP_HEAD and "function" not in stripped[:40]:
            i += 1
            continue

        name, sig, kind = None, stripped, None
        m = FN_PREFIX_RE.search(line[:200])
        if m and m.group(1) not in SKIP_HEAD:
            name, kind = m.group(1) or "<anonymous>", "fn"
        if name is None:
            m = ARROW_RE.match(stripped)
            if m:
                name, kind, sig = m.group(1), "arrow", stripped
        if name is None:
            m = DECL_RE.match(stripped)
            if m and m.group(1) not in SKIP_HEAD and m.group(1) not in CALLBACK_NAMES:
                before = stripped[:m.start(1)].rstrip()
                if not before.endswith("."):
                    name, kind, sig = m.group(1), "method", stripped
        if name is None:
            i += 1
            continue

        # bound the body by brace balance
        depth, end = 0, i
        opened = "{" in line
        for k in range(i, n):
            depth += code_lines[k].count("{") - code_lines[k].count("}")
            if opened and depth <= 0:
                end = k
                break
            if not opened:  # one-line arrow / macro
                end = i
                break
        if opened and end == i and depth > 0:
            end = n - 1
        if end - i > 4000:
            end = i + 4000
        funcs.append({"name": name, "start": i, "end": end, "params": extract_params(sig, lang),
                      "signature": " ".join(sig.split())[:150], "indent": len(line) - len(line.lstrip())})
        i += 1
    return funcs


# --------------------------------------------------------------------------------------
# Violation patterns
# --------------------------------------------------------------------------------------
DEBUG_PATTERNS = {
    "js": re.compile(r"\bconsole\s*\.\s*(log|debug|info|trace|warn|error|table|dir|count)\s*\("),
    "ts": re.compile(r"\bconsole\s*\.\s*(log|debug|info|trace|warn|error|table|dir|count)\s*\("),
    "java": re.compile(r"\bSystem\s*\.\s*(out|err)\s*\.\s*print\w*\s*\("),
    "kotlin": re.compile(r"\bprintln\s*\(|\bSystem\s*\.\s*out\s*\."),
    "csharp": re.compile(r"\bConsole\s*\.\s*Write\w*\s*\("),
    "go": re.compile(r"\bfmt\s*\.\s*Print\w*\s*\("),
    "python": re.compile(r"^\s*print\s*\(|[^\w.]print\s*\("),
    "ruby": re.compile(r"^\s*(?:puts|p|print)\s+|^\s*(?:puts|print)\s*\("),
    "php": re.compile(r"\b(?:var_dump|print_r|error_log)\s*\("),
}
NEGATIVE_RE = re.compile(
    r"\bif\s*\(\s*![^)]*[&|]{2}[^)]*!"          # double negation: negate-a and negate-b
    r"|\bif\s*\(\s*!\s*\w*(?:Not|No|Non|Un|Disable|Missing|Invalid)"   # negation of a negative name
    r"|\bif\s+not\s+\w+\s+(?:and|or)\s+not\b"   # python: not a and not b
)
NUM_RE = re.compile(r"(?<![\w.$])(?:0[xXbBoO][0-9a-fA-F_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?)"
                    r"[lLuUlLfFdD]?(?![\w.])")
# "constant declaration" = a line assigning to an UPPER_SNAKE name (or a const/final
# declaration in Go/Rust). That is the *solution* to a magic number, not a violation.
UPPER_ASSIGN_RE = re.compile(r"^\s*(?:(?:export|public|private|protected|static|final|const|let|var|"
                             r"readonly|val|def|new)\s+)*([A-Z][A-Z0-9_]*)\s*[:=]")
KEYWORD_CONST_RE = re.compile(r"\b(?:constexpr|static\s+final|public\s+static\s+final|final\s+[A-Z]"
                              r"[A-Z0-9_]*\s*=)\b")
LANG_CONST_DECL_RE = re.compile(r"^\s*(?:var|const|let\s+mut|let|static)\s+[A-Za-z_]\w*\s*"
                                r"(?::\s*[\w<>\[\]]+)?\s*=")
CONST_LANGS = {"go", "rust", "c", "cpp"}
CASE_LABEL_RE = re.compile(r"^\s*case\s+")
DICT_VALUE_RE = re.compile(r"^\s*[\x27\"`][\w./@-]+[\x27\"`]\s*:")


def is_constant_line(code: str, lang: str) -> bool:
    if UPPER_ASSIGN_RE.match(code) or KEYWORD_CONST_RE.search(code):
        return True
    return lang in CONST_LANGS and bool(LANG_CONST_DECL_RE.match(code))
COMMENT_CODE_RE = re.compile(
    r"^(?:(?://|#|\*|/\*)\s*)*"
    r"(?:if\s*\(|for\s*\(|while\s*\(|return\b|const\s|let\s|var\s|function\s|def\s|class\s|"
    r"import\s|export\s|throw\s|await\s|console\.|System\.|fmt\.|print\(|}\s*;?|\{\s*$|"
    r"[\w.$]+\([^)]*\)\s*;|[\w.$]+\s*=[^=~!<>])")
TODO_RE = re.compile(r"\b(TODO|FIXME|XXX|HACK|BUG|WIP)\b\s*[:(]?")
ALLOW_LINE_RE = re.compile(r"(?://|#|/\*)\s*cc-scan:allow\s+([A-Za-z_,\s]+).*$")
ALLOW_FILE_RE = re.compile(r"(?://|#|/\*)\s*cc-scan:allow-file\s+([A-Za-z_,\s]+).*$")


def looks_like_code(text: str) -> bool:
    t = text.strip()
    if len(t) < 5 or not COMMENT_CODE_RE.match(t):
        return False
    low = t.lower()
    if low.startswith(("todo", "fixme", "note:", "warning:", "deprecated", "see ", "http")):
        return False
    return True


def indent_of(line: str) -> int:
    """Indentation width of a code line (string placeholders kept intact)."""
    return len(line) - len(line.lstrip())


def find_empty_catch(code_lines: list[str], comment_lines: list[str], lang: str) -> list[int]:
    """Returns the 0-based line indexes where catch/except swallows an error."""
    hits: list[int] = []
    n = len(code_lines)
    filler = ("pass", "...", "noop", "pass  # noqa", "pass  # best effort")
    if lang in PYTHONISH:
        rx = re.compile(r"^\s*(?:except|finally)\b[^:]*:(?P<rest>\s*(?:#.*)?)$")
        rx_inline = re.compile(r"^\s*(?:except|finally)\b[^:]*:\s*(?P<body>pass|continue|\.\.\.)\s*$")
        for i, line in enumerate(code_lines):
            s = real(line)
            if rx_inline.match(s):
                hits.append(i)
                continue
            m = rx.match(s)
            if not m:
                continue
            ind = indent_of(line)
            j, body = i + 1, []
            while j < n:
                raw_cur = code_lines[j]
                cur = real(raw_cur)
                if not cur:
                    j += 1
                    continue
                if indent_of(raw_cur) <= ind:
                    break
                body.append(cur.strip())
                j += 1
            documented = any(comment_lines[k] for k in range(i + 1, j))
            if not documented and (not body
                                   or all(b.split("#")[0].strip().rstrip(";") in filler for b in body)):
                hits.append(i)
        return hits

    rx = re.compile(r"\b(?:catch|rescue)\b")
    for i, line in enumerate(code_lines):
        s = real(line)
        if not rx.search(s) or "{" not in line:
            continue
        depth = line.count("{") - line.count("}")
        if depth <= 0:
            body = line.split("{", 1)[1].split("}")[0] if "{" in line else ""
            if not real(body).strip():
                hits.append(i)
            continue
        j, body_lines = i + 1, []
        while j < n and depth > 0:
            depth += code_lines[j].count("{") - code_lines[j].count("}")
            body_lines.append(real(code_lines[j]).strip("{} ;"))
            j += 1
        documented = any(comment_lines[k] for k in range(i + 1, j))
        if not [b for b in body_lines if b] and not documented:
            hits.append(i)
    return hits


# --------------------------------------------------------------------------------------
# Scan one file
# --------------------------------------------------------------------------------------
def scan_file(path: Path, root: Path, cfg: dict, is_test: bool) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:  # pragma: no cover
        return [], {"error": str(exc)}
    if "\x00" in text[:4096]:
        return [], {"binary": True}

    rel = path.relative_to(root).as_posix() if _relative_ok(path, root) else path.name
    lang = detect_lang(path)
    if lang is None:
        return [], {"skipped": True}

    lines = text.split("\n")
    code_lines, comment_lines = preprocess(text, lang)
    skip = set(cfg.get("skipRules") or [])
    if is_test:
        skip |= set(cfg.get("skipRulesForTests") or [])

    allow_lines: dict[int, set[str]] = {}
    allow_file: set[str] = set()
    for idx, src_line in enumerate(lines, start=1):
        m = ALLOW_LINE_RE.search(src_line)
        if m:
            allow_lines.setdefault(idx, set()).update(
                x.strip().upper() for x in m.group(1).split(",") if x.strip())
        m = ALLOW_FILE_RE.search(src_line)
        if m and idx <= 30:
            allow_file.update(x.strip().upper() for x in m.group(1).split(",") if x.strip())

    def add(rule: str, line_no: int, message: str, value=None, hint: str = "") -> None:
        if rule in skip or rule in allow_file or "ALL" in allow_file:
            return
        allowed_here = allow_lines.get(line_no) or set()
        if rule in FUNCTION_RULES:
            # function-level rules report at the declaration line: an allow note may sit right above it
            allowed_here |= allow_lines.get(line_no - 1) or set()
        else:
            # every other rule: the allow comment covers its own line AND the line directly below it
            allowed_here |= allow_lines.get(line_no - 1) or set()
        if rule in allowed_here or "ALL" in allowed_here:
            return
        sev = RULES.get(rule, ("", "warning"))[1]
        findings.append({"rule": rule, "severity": sev, "file": rel, "line": line_no,
                         "message": message, "value": value, "hint": hint})

    # 1) comment rules ---------------------------------------------------------------
    commented_lines: set[int] = set()
    for idx, comment in enumerate(comment_lines, start=1):
        raw = lines[idx - 1]
        if comment and looks_like_code(comment):
            commented_lines.add(idx)
            add("COMMENTED_CODE", idx, f"Commented-out code: {comment[:80]}",
                hint="Delete it - Git keeps the history; if it must be remembered, open an issue.")
        m = TODO_RE.search(comment or "")
        if m:
            add("TODO_MARK", idx, f"Technical debt: {(comment or raw).strip()[:80]}",
                hint="Turn it into an issue with an ID and an owner.")

    block_open = None
    for idx, raw in enumerate(lines, start=1):
        if raw.lstrip().startswith("/*") and block_open is None:
            block_open = idx
        if block_open is not None and raw.rstrip().endswith("*/"):
            if idx - block_open + 1 >= 6:
                add("BLOCK_COMMENT", block_open,
                    f"Comment block {idx - block_open + 1} lines - consider moving it to README/ADR")
            block_open = None

    # 2) format rules ----------------------------------------------------------------
    max_line = int(cfg.get("maxLineLength", 120))
    for idx, raw in enumerate(lines, start=1):
        if len(raw) > max_line and idx not in commented_lines and not raw.lstrip().startswith(
                ("http://", "https://", " * ", "* ")):
            add("LINE_TOO_LONG", idx, f"Line is {len(raw)} characters (limit {max_line})",
                value=len(raw), hint="Break the line: one parameter/argument per line, one indent level.")
        if raw.rstrip() != raw and raw.strip():
            add("TRAILING_WHITESPACE", idx, "Trailing whitespace at end of line")
        indent_part = raw[:len(raw) - len(raw.lstrip())] if raw.strip() else ""
        if "\t" in indent_part and " " in indent_part:
            add("MIXED_INDENT", idx, "Mixed tabs and spaces",
                hint="Pick one style via .editorconfig + a formatter (prettier/black/gofmt).")

    # 3) debug / negative condition ---------------------------------------------------
    pat = DEBUG_PATTERNS.get(lang)
    if pat:
        for idx, code in enumerate(code_lines, start=1):
            if pat.search(code):
                add("DEBUG_STATEMENT", idx, f"Leftover debug log: {lines[idx-1].strip()[:70]}",
                    hint="Use a logger with a log level, or delete it before merge.")
    for idx, code in enumerate(code_lines, start=1):
        if NEGATIVE_RE.search(code):
            add("NEGATIVE_CONDITIONAL", idx,
                "Negated condition combined with a boolean operator: rename to the positive form",
                hint="VD: if (!isActive && !isBlocked) -> if (isBlockedOrInactive)")

    # 4) magic number ------------------------------------------------------------------
    allowed = {str(x) for x in cfg.get("magicNumbersAllowed", [])}
    for idx, code in enumerate(code_lines, start=1):
        src = lines[idx - 1]
        if is_constant_line(code, lang) or CASE_LABEL_RE.match(real(src)):
            continue
        if DICT_VALUE_RE.match(src):         # {"maxLineLength": 120} - self-documenting
            continue
        for m in NUM_RE.finditer(code):
            num = m.group(0).rstrip("lLuUlLfFdD")
            if num in allowed or num.replace("_", "") in allowed:
                continue
            before, after = code[:m.start()], code[m.end():]
            if re.search(r"[\[(<]\s*$", before) and re.match(r"\s*[\])>]", after):
                continue                       # sizes and indexes: arr[0], len(..., 1)
            if re.search(r"(?:length|size|count|index|offset|limit|capacity)\s*[:.=<>]=?\s*$",
                         before, re.I):
                continue
            add("MAGIC_NUMBER", idx, f"Magic number `{num}` in: `{src.strip()[:60]}`",
                value=num, hint="Extract a named constant or a named parameter.")

    # 5) empty catch ---------------------------------------------------------------------
    for idx in find_empty_catch(code_lines, comment_lines, lang):
        add("EMPTY_CATCH", idx + 1,
            f"`{lines[idx].strip()[:70]}` - the catch does nothing, the error is swallowed",
            hint="Log and rethrow (wrapped), or handle it explicitly with a reason comment.")

    # 6) functions -------------------------------------------------------------------------
    funcs = find_functions(code_lines, lang)
    max_nesting = int(cfg.get("maxNesting", 3))
    for fn in funcs:
        body = fn["end"] - fn["start"] + 1
        if body > int(cfg.get("hardFunctionLines", 70)):
            add("HUGE_FUNCTION", fn["start"] + 1,
                f"function `{fn['name']}` is {body} lines (must split above "
                f"{cfg.get('hardFunctionLines', 70)})", value=body,
                hint="Split by business step; one function = one level of abstraction.")
        elif body > int(cfg.get("maxFunctionLines", 40)):
            add("LONG_FUNCTION", fn["start"] + 1,
                f"function `{fn['name']}` is {body} lines (recommended <= "
                f"{cfg.get('maxFunctionLines', 40)})", value=body,
                hint="Extract the steps into their own functions, named in business language.")

        params = fn.get("params")
        if params:
            hard, soft = int(cfg.get("hardParams", 5)), int(cfg.get("maxParams", 3))
            if len(params) > hard:
                add("HARD_PARAMS", fn["start"] + 1,
                    f"`{fn['name']}` takes {len(params)} parameters", value=len(params),
                    hint="Use an options/command object (or a dataclass/record).")
            elif len(params) > soft:
                add("TOO_MANY_PARAMS", fn["start"] + 1,
                    f"`{fn['name']}` takes {len(params)} parameters (recommended <= {soft})",
                    value=len(params), hint="Group related parameters into one named object.")
            for p in params:
                pname = re.split(r"[:=\s*,)]", p.strip())[0].lstrip("*&")
                if re.match(r"^(is|has|should|can|need|allow|enable|disable|skip|force|with)[A-Z_]",
                            pname):
                    add("BOOLEAN_PARAM", fn["start"] + 1,
                        f"`{fn['name']}(..., {pname}, ...)` takes a boolean flag",
                        hint="Split into two functions, or replace the flag with an enum/flag object.")

        # nesting + complexity
        depth, max_depth, complexity = 0, 0, 1
        if lang in PYTHONISH:
            base = fn["indent"]
            for k in range(fn["start"] + 1, fn["end"] + 1):
                cur = code_lines[k]
                if not real(cur):
                    continue
                ind = len(cur[:len(cur) - len(cur.lstrip())].expandtabs(4))
                max_depth = max(max_depth, (ind - base) // 4)
                body_txt = real(cur)
                complexity += len(re.findall(r"\b(?:if|elif|for|while|except|and|or|with|assert)\b",
                                             body_txt)) + body_txt.count(" if ")
        else:
            for k in range(fn["start"], fn["end"] + 1):
                cur = code_lines[k]
                if k > fn["start"]:
                    max_depth = max(max_depth, depth - 1)
                depth += cur.count("{") - cur.count("}")
                if not real(cur):
                    continue
                body_txt = real(cur)
                complexity += len(re.findall(r"\b(?:if|for|while|case|catch|elif)\b", body_txt))
                complexity += body_txt.count("&&") + body_txt.count("||")
                complexity += body_txt.count("?") - body_txt.count("?:")
        soft_c, hard_c = int(cfg.get("maxComplexity", 10)), int(cfg.get("hardComplexity", 15))
        if complexity > hard_c:
            add("HARD_COMPLEXITY", fn["start"] + 1,
                f"`{fn['name']}` has ~{complexity} logical branches (hard limit {hard_c})",
                value=complexity, hint="Table-driven or strategy, or a map/filter/reduce pipeline.")
        elif complexity > soft_c:
            add("COMPLEXITY", fn["start"] + 1,
                f"`{fn['name']}` has ~{complexity} logical branches (recommended <= {soft_c})",
                value=complexity, hint="Guard clauses + one function per branch.")
        if max_depth > max_nesting:
            add("DEEP_NESTING", fn["start"] + 1,
                f"`{fn['name']}` nests {max_depth} levels deep (recommended <= {max_nesting})",
                value=max_depth, hint="Invert conditions + return early; extract the core into its own function.")

    meta = {"lang": lang, "lines": len(lines), "funcs": len(funcs),
            "loc": sum(real(c) and 1 or 0 for c in code_lines),
            "commentLines": sum(1 for c in comment_lines if c)}
    return findings, meta


def _relative_ok(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


# --------------------------------------------------------------------------------------
# DRY: find duplicated blocks
# --------------------------------------------------------------------------------------
def find_duplicates(root: Path, files: list[Path], cfg: dict, test_map: dict) -> list[dict]:
    size = int(cfg.get("dupMinLines", 8) or 0)
    if size < 4:
        return []
    windows: dict[str, list[tuple[str, int]]] = defaultdict(list)
    for path in files:
        if test_map.get(str(path)):
            continue
        lang = detect_lang(path)
        if lang is None or not _relative_ok(path, root):
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        code_lines, _ = preprocess(text, lang)
        norm = []
        for idx, line in enumerate(code_lines, start=1):
            s = re.sub(r"\s+", " ", real(line))
            if len(s) >= 4:
                norm.append((idx, s))
        for start in range(0, max(0, len(norm) - size + 1)):
            chunk = norm[start:start + size]
            key = "\n".join(s for _, s in chunk)
            if key.count("(") < 2 or key.count("\n") < size - 1:
                continue
            windows[key].append((path.relative_to(root).as_posix(), chunk[0][0]))

    out, seen, covered = [], set(), defaultdict(list)
    for key, locs in sorted(windows.items()):
        files_involved = {f for f, _ in locs}
        far = sorted(locs)
        spread = any(b[1] - a[1] > size * 4 for a, b in zip(far, far[1:])) if len(far) > 1 else False
        if len(locs) < 2 or not (len(files_involved) > 1 or spread):
            continue
        digest = hash(key)
        if digest in seen:
            continue
        seen.add(digest)
        anchor_file, anchor_line = far[0]
        if any(anchor_line <= until for until in covered[anchor_file]):
            continue
        covered[anchor_file].append(anchor_line + size * 2 - 1)
        rest = ", ".join(f"{f}:{l}" for f, l in far[1:4])
        out.append({"rule": "DUPLICATE_BLOCK", "severity": RULES["DUPLICATE_BLOCK"][1],
                    "file": far[0][0], "line": far[0][1], "value": size,
                    "message": f"{size}-line block repeated in {len(locs)} places ({rest})",
                    "hint": "Move it into a shared function or strategy - but do not merge two things that only happen to look alike."})
    return out


# --------------------------------------------------------------------------------------
# Repository, config and baseline handling
# --------------------------------------------------------------------------------------
def iter_files(root: Path, cfg: dict):
    ignore = ALWAYS_IGNORE_DIRS | set(cfg.get("ignoreDirs") or [])
    globs = list(cfg.get("ignoreGlobs") or [])
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(d for d in dirnames if d not in ignore and not d.startswith("."))
        for name in sorted(filenames):
            path = Path(dirpath) / name
            if any(fnmatch.fnmatch(name, g) for g in globs):
                continue
            if detect_lang(path) is None:
                continue
            try:
                if path.stat().st_size > 1_500_000:
                    continue
            except OSError:
                continue
            yield path


def load_config(explicit: str | None, roots: list[Path]) -> dict:
    cfg = dict(DEFAULTS)
    candidates = [Path(explicit)] if explicit else []
    for r in roots:
        base = r if r.is_dir() else r.parent
        candidates += [base / "clean-code.config.json", base / ".clean-code.json"]
    for cand in candidates:
        if cand.is_file():
            try:
                loaded = json.loads(cand.read_text(encoding="utf-8"))
                # ignoreDirs is EXTENDED, never replaced: a config may only *add* folders to skip, so
                # a short list can never switch off the built-in protections.
                if isinstance(loaded.get("ignoreDirs"), list):
                    loaded["ignoreDirs"] = sorted(set(DEFAULTS["ignoreDirs"]) | set(loaded["ignoreDirs"]))
                cfg.update(loaded)
                cfg["_configPath"] = str(cand)
            except json.JSONDecodeError as exc:
                print(f"[cc-scan] {cand} is not valid JSON: {exc}", file=sys.stderr)
            break
    return cfg


def baseline_path(root: Path) -> Path:
    for cand in [root, *list(root.parents)[:3]]:
        p = cand / ".clean-code-baseline.json"
        if p.is_file():
            return p
    return root / ".clean-code-baseline.json"


def load_baseline(root: Path) -> dict:
    path = baseline_path(root)
    if not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        counts = data.get("counts", data) if isinstance(data, dict) else {}
        return {k: v for k, v in counts.items() if isinstance(v, dict)}
    except (json.JSONDecodeError, OSError):
        return {}


def write_baseline(root: Path, findings: list[dict]) -> Path:
    counts: dict[str, dict[str, int]] = {}
    for f in findings:
        counts.setdefault(f["rule"], {})
        counts[f["rule"]][f["file"]] = counts[f["rule"]].get(f["file"], 0) + 1
    path = root / ".clean-code-baseline.json"
    path.write_text(json.dumps({"version": VERSION, "counts": counts}, indent=2,
                              ensure_ascii=False) + "\n", encoding="utf-8")
    return path


def apply_baseline(findings: list[dict], baseline: dict) -> tuple[list[dict], int]:
    if not baseline:
        return findings, 0
    remaining = {rule: dict(files) for rule, files in baseline.items()}
    kept, suppressed = [], 0
    for f in sorted(findings, key=lambda x: (-SEVERITY_ORDER[x["severity"]], x["file"], x["line"])):
        bucket = remaining.get(f["rule"])
        if bucket and bucket.get(f["file"], 0) > 0:
            bucket[f["file"]] -= 1
            suppressed += 1
            continue
        kept.append(f)
    return kept, suppressed


def grade(score: float) -> str:
    return ("A" if score >= 90 else "B" if score >= 80 else "C" if score >= 70
            else "D" if score >= 55 else "E")


# --------------------------------------------------------------------------------------
# Main
# --------------------------------------------------------------------------------------
def _quiet_broken_pipe() -> None:
    """`cc-scan | head` must stay silent like grep, not dump a traceback when the reader closes early.

    On POSIX: restore the default SIGPIPE (the process dies quietly, the shell sees 141 like any CLI).
    Windows has no SIGPIPE - skip it there, output always drains.
    """
    if hasattr(signal, "SIGPIPE"):
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)


def main(argv: list[str] | None = None) -> int:
    _quiet_broken_pipe()
    ap = argparse.ArgumentParser(prog="cc-scan",
                                 description="Scan a repo for clean-code violations (heuristic, nothing to install).")
    ap.add_argument("paths", nargs="*", default=["."], help="file or directory to scan")
    ap.add_argument("--config", help="path to clean-code.config.json")
    ap.add_argument("--json", action="store_true", help="print the JSON report")
    ap.add_argument("-o", "--output", help="write the JSON report to a file")
    ap.add_argument("--fail-on", choices=["none", "info", "warning", "error"], default="error",
                    help="exit 1 when a finding at or above this level exists (default: error)")
    ap.add_argument("--max-line", type=int, dest="maxLineLength")
    ap.add_argument("--max-fn-lines", type=int, dest="maxFunctionLines")
    ap.add_argument("--max-params", type=int, dest="maxParams")
    ap.add_argument("--max-nesting", type=int, dest="maxNesting")
    ap.add_argument("--no-dup", action="store_true", help="disable duplicate detection")
    ap.add_argument("--no-baseline", dest="use_baseline", action="store_false", default=True,
                    help="ignore .clean-code-baseline.json")
    ap.add_argument("--update-baseline", action="store_true",
                    help="freeze current findings into .clean-code-baseline.json")
    ap.add_argument("--list-rules", action="store_true")
    ap.add_argument("--version", action="store_true")
    args = ap.parse_args(argv)

    if args.version:
        print(f"cc-scan {VERSION}")
        return 0
    if args.list_rules:
        print(f"{'LEVEL':<8} {'RULE':<20} DESCRIPTION")
        for rule, (desc, sev) in RULES.items():
            print(f"{sev:<8} {rule:<20} {desc}")
        return 0

    targets = [Path(p).resolve() for p in (args.paths or ["."])]
    missing = [str(t) for t in targets if not t.exists()]
    for m in missing:
        print(f"[cc-scan] not found: {m}", file=sys.stderr)
    targets = [t for t in targets if t.exists()]
    if not targets:
        return 2
    root = targets[0] if len(targets) == 1 and targets[0].is_dir() else Path.cwd()

    # Config is loaded from the scanned directory only (or cwd when several paths are passed).
    # Picking up a child folder's config by accident would be unpredictable behaviour.
    cfg = load_config(args.config, [root])
    for key in ("maxLineLength", "maxFunctionLines", "maxParams", "maxNesting"):
        if getattr(args, key, None):
            cfg[key] = getattr(args, key)
    if args.no_dup:
        cfg["dupMinLines"] = 0

    files: list[Path] = []
    for t in targets:
        files.extend([t] if t.is_file() else list(iter_files(t, cfg)))
    files = sorted(set(files))

    test_res = [re.compile(p) for p in cfg.get("testPathPatterns", [])]
    test_map, findings, metas = {}, [], {}
    for path in files:
        rel = path.relative_to(root).as_posix() if _relative_ok(path, root) else path.name
        is_test = any(rx.search(rel) for rx in test_res)
        test_map[str(path)] = is_test
        f, meta = scan_file(path, root, cfg, is_test)
        findings.extend(f)
        if not meta.get("skipped") and not meta.get("error") and not meta.get("binary"):
            metas[rel] = meta

    findings.extend(find_duplicates(root, files, cfg, test_map))

    src_total = len(metas)
    test_total = sum(1 for p, t in test_map.items() if t and detect_lang(Path(p)))
    if src_total and test_total / src_total < float(cfg.get("minTestRatio", 0.15)):
        findings.append({"rule": "LOW_TEST_RATIO", "severity": RULES["LOW_TEST_RATIO"][1],
                         "file": "(project)", "line": 0, "value": round(test_total / src_total, 3),
                         "message": f"{test_total} test files / {src_total} source files "
                                    f"({test_total / src_total:.0%}), recommended >= "
                                    f"{float(cfg.get('minTestRatio', 0.15)):.0%}",
                         "hint": "At least one test per new business unit; start where the code changes most."})

    findings.sort(key=lambda x: (x["file"], x["line"], x["rule"]))
    findings, suppressed = apply_baseline(findings, load_baseline(root) if args.use_baseline else {})

    if args.update_baseline:
        path = write_baseline(root, findings)
        print(f"[cc-scan] baseline frozen: {len(findings)} findings -> "
              f"{path.relative_to(root) if _relative_ok(path, root) else path}")
        if not args.use_baseline:
            # --no-baseline with --update-baseline: write only, still report everything
            pass
        else:
            findings, more = apply_baseline(findings, load_baseline(root))
            suppressed += more
            counts = defaultdict(int)
            for f in findings:
                counts[f["severity"]] += 1
                counts[f["rule"]] += 1
            score = max(0.0, min(100.0, 100 - (counts["error"] * 4 + counts["warning"]
                                               + counts["info"] * 0.25)))

    counts: dict[str, int] = defaultdict(int)
    for f in findings:
        counts[f["severity"]] += 1
        counts[f["rule"]] += 1
    score = max(0.0, min(100.0, 100 - (counts["error"] * 4 + counts["warning"]
                                       + counts["info"] * 0.25)))
    report = {
        "tool": "cc-scan", "version": VERSION, "root": str(root),
        "config": {k: cfg[k] for k in ("maxLineLength", "maxFunctionLines", "maxParams",
                                        "maxNesting", "maxComplexity", "dupMinLines")},
        "filesScanned": len(files), "filesWithFindings": len({f["file"] for f in findings}),
        "score": round(score, 1), "grade": grade(score),
        "counts": dict(counts), "suppressedByBaseline": suppressed,
        "totals": {"sourceLoc": sum(m["loc"] for m in metas.values()),
                   "functions": sum(m["funcs"] for m in metas.values())},
        "findings": findings,
    }

    out = json.dumps(report, indent=2, ensure_ascii=False)
    if args.output:
        Path(args.output).write_text(out + "\n", encoding="utf-8")
        print(f"[cc-scan] wrote report: {args.output}")
    if args.json:
        print(out)
    elif not args.output:
        print_report(report, root)
    elif not args.json:
        print_report(report, root)

    threshold = SEVERITY_ORDER.get(args.fail_on, 99)
    worst = max((SEVERITY_ORDER[f["severity"]] for f in findings), default=0)
    return 1 if (args.fail_on != "none" and worst >= threshold) else 0


def print_report(report: dict, root: Path) -> None:
    c = report["counts"]
    print(f"cc-scan v{report['version']} · {report['filesScanned']} file · {report['root']}")
    print(f"Clean Code score: {report['score']}/100 (grade {report['grade']})  "
          f"· error={c.get('error', 0)} warning={c.get('warning', 0)} info={c.get('info', 0)}")
    if report["suppressedByBaseline"]:
        print(f"(ignored {report['suppressedByBaseline']} findings frozen in the baseline)")
    if not report["findings"]:
        print("\n✅ No findings under the current configuration.")
        return
    grouped: dict[str, list[dict]] = defaultdict(list)
    for f in report["findings"]:
        grouped[f["file"]].append(f)

    def worst_of(items: list[dict]) -> int:
        return max(SEVERITY_ORDER[x["severity"]] for x in items)

    print()
    for file in sorted(grouped, key=lambda k: (-worst_of(grouped[k]), k)):
        items = sorted(grouped[file], key=lambda x: (x["line"], x["rule"]))
        print(f"── {file}")
        for f in items[:40]:
            print(f"   {f['line']:>5}  {f['severity'][:4]:<5} {f['rule']:<20} {f['message']}")
            if f.get("hint"):
                print(f"          ↳ {f['hint']}")
        if len(items) > 40:
            print(f"   ... {len(items) - 40} more findings in this file")
    print("\nSummary by rule:")
    for rule, n in sorted(((k, v) for k, v in c.items() if k in RULES), key=lambda kv: -kv[1]):
        print(f"   {n:>4}  {rule:<20} {RULES[rule][0]}")
    print("\nCI suggestion: --fail-on error | --json -o cc.json | --update-baseline for legacy code")


if __name__ == "__main__":
    raise SystemExit(main())
