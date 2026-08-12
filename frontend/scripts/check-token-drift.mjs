#!/usr/bin/env node
/* eslint-disable no-console */

// Design token contract guard (P3.1 / P3.2).
//
// Verifies globals.css + components stay within the locked token contract:
//
//   canonical semantic layer (--surface-* / --text-* / --border-* / --accent-* / --state-* / --elevation-*)
//           ↓ aliases only, never raw values
//   shadcn compatibility layer (--background, --popover, ...)
//
// Checks:
//   1. Shadcn compatibility layer aliases canonical tokens — no raw color
//      values (allowed primitives: --radius, --chart-*, *-foreground, --overlay).
//   2. Every canonical color token is defined in BOTH :root and [data-theme="dark"].
//   3. Canonical color values match the recorded snapshot (light + dark) —
//      a rename is value-preserving; this catches accidental theme drift.
//      Update SNAPSHOT deliberately when changing theme values.
//   4. No --app-* COLOR tokens anywhere in src/ components (the --app-* prefix
//      is reserved for layout metrics only).
//   5. Components do not reference raw shadcn semantic vars (var(--primary),
//      var(--info), ...) — use canonical tokens or shadcn utility classes.
//
// Exit 0 = clean, Exit 1 = contract violation, Exit 2 = structural error.

import { readFileSync, readdirSync, statSync } from "node:fs";
import { resolve, dirname, join, extname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");
const cssPath = join(root, "src/styles/globals.css");
const css = readFileSync(cssPath, "utf-8");

/* ── Snapshot: canonical color values per theme (2026-08-12, migration F1) ──
 * The migration renamed --app-* → canonical names WITHOUT changing values.
 * These maps pin those values so any future theme drift fails CI. Update them
 * deliberately (and in the same commit as the theme change). */
const LIGHT = {
  "surface-app": "#f0f2f5",
  "surface-nav": "#f5f7fa",
  "surface-panel": "#f8fafc",
  "surface-editor": "#ffffff",
  "surface-floating": "#ffffff",
  "surface-hover": "#f1f5f9",
  "surface-active": "#e2e8f0",
  "text-primary": "#0f172a",
  "text-secondary": "#475569",
  "text-tertiary": "#94a3b8",
  "border-subtle": "rgba(0, 0, 0, 0.06)",
  "border-default": "rgba(0, 0, 0, 0.12)",
  "border-strong": "rgba(0, 0, 0, 0.18)",
  accent: "#6366f1",
  "accent-hover": "#4f46e5",
  "accent-soft": "rgba(99, 102, 241, 0.1)",
  "accent-foreground": "#ffffff",
  "state-success": "#059669",
  "state-warning": "#d97706",
  "state-danger": "#ef4444",
  "state-info": "#3b82f6",
  "elevation-lg": "0 8px 32px rgba(0, 0, 0, 0.1)",
  "elevation-popover": "0 4px 24px rgba(0, 0, 0, 0.12)",
};

const DARK = {
  "surface-app": "#090d14",
  "surface-nav": "#0c121c",
  "surface-panel": "#0e131b",
  "surface-editor": "#10151d",
  "surface-floating": "#151d2e",
  "surface-hover": "rgba(148, 163, 184, 0.06)",
  "surface-active": "rgba(148, 163, 184, 0.1)",
  "text-primary": "#e6edf7",
  "text-secondary": "#93a4ba",
  "text-tertiary": "#66778f",
  "border-subtle": "rgba(148, 163, 184, 0.08)",
  "border-default": "rgba(148, 163, 184, 0.14)",
  "border-strong": "rgba(148, 163, 184, 0.22)",
  accent: "#5b7cff",
  "accent-hover": "#6d8aff",
  "accent-soft": "rgba(91, 124, 255, 0.14)",
  "accent-foreground": "#ffffff",
  "state-success": "#39d98a",
  "state-warning": "#f5b942",
  "state-danger": "#f05d6f",
  "state-info": "var(--accent)",
  "elevation-lg": "0 8px 32px rgba(0, 0, 0, 0.55)",
  "elevation-popover": "0 4px 24px rgba(0, 0, 0, 0.5)",
};

/* ── Removed --app-* COLOR tokens (must never reappear in components) ── */
const BANNED_APP_COLOR_TOKENS = [
  "--app-surface-0",
  "--app-surface-1",
  "--app-surface-2",
  "--app-surface-3",
  "--app-surface-4",
  "--app-hover",
  "--app-active",
  "--app-border",
  "--app-border-subtle",
  "--app-border-strong",
  "--app-text",
  "--app-text-muted",
  "--app-text-dim",
  "--app-primary",
  "--app-primary-hover",
  "--app-primary-soft",
  "--app-success",
  "--app-warning",
  "--app-danger",
  "--app-shadow-lg",
  "--app-shadow-popover",
];

/* ── Raw shadcn semantic vars — components must use canonical tokens ── */
const SHADCN_SEMANTIC_VARS = [
  "--background",
  "--foreground",
  "--card",
  "--card-foreground",
  "--popover",
  "--popover-foreground",
  "--primary",
  "--primary-foreground",
  "--secondary",
  "--secondary-foreground",
  "--muted",
  "--muted-foreground",
  // NOTE: --accent / --accent-foreground are CANONICAL brand tokens now
  // (shadcn's old --accent hover semantics moved to the @theme inline mapping).
  "--destructive",
  "--destructive-foreground",
  "--success",
  "--success-foreground",
  "--warning",
  "--warning-foreground",
  "--info",
  "--border",
  "--input",
  "--ring",
  "--sidebar",
  "--sidebar-foreground",
  "--sidebar-primary",
  "--sidebar-primary-foreground",
  "--sidebar-accent",
  "--sidebar-accent-foreground",
  "--sidebar-border",
  "--sidebar-ring",
];

const ALLOWED_PROP_PREFIXES = ["--radius", "--chart-"];
const ALLOWED_PROP_SUFFIXES = ["-foreground"];
const ALLOWED_PROPS_EXACT = new Set(["--overlay"]);

const RAW_COLOR_RE = /#[0-9a-fA-F]{3,8}\b|rgba?\(|oklch\(|hsla?\(/;

let drift = 0;
const fail = (msg) => {
  console.error(`DRIFT: ${msg}`);
  drift++;
};

/* ── 1. Shadcn compatibility layer: aliases only ─────────────────────── */

const compatStart = css.indexOf("shadcn compatibility layer");
const canonStart = css.indexOf("Canonical design tokens");

if (compatStart === -1 || canonStart === -1 || canonStart <= compatStart) {
  console.error(
    "Could not find shadcn compatibility layer / canonical design tokens boundaries in globals.css",
  );
  process.exit(2);
}

const compatBlock = css.slice(compatStart, canonStart);
for (const line of compatBlock.split("\n")) {
  const trimmed = line.trim();
  if (!trimmed || trimmed.startsWith("/*") || trimmed.startsWith("*") || trimmed.startsWith("//")) {
    continue;
  }

  const declMatch = trimmed.match(/^(--[\w-]+):\s*([^;]+);/);
  if (!declMatch) continue;

  const [, prop, value] = declMatch;

  if (ALLOWED_PROP_PREFIXES.some((p) => prop.startsWith(p))) continue;
  if (ALLOWED_PROP_SUFFIXES.some((s) => prop.endsWith(s))) continue;
  if (ALLOWED_PROPS_EXACT.has(prop)) continue;
  if (value.startsWith("var(")) continue;

  if (RAW_COLOR_RE.test(value)) {
    fail(`${prop} has raw value "${value}" in shadcn compatibility layer`);
    console.error(
      `  → Should alias to a canonical token (--surface-*/--text-*/--border-*/--accent-*/--state-*) instead`,
    );
  }
}

/* ── 2 + 3. Canonical layer: theme completeness + value snapshot ─────── */

function parseThemeBlock(block) {
  const tokens = {};
  for (const line of block.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("/*") || trimmed.startsWith("*")) continue;
    // value runs to the first ";" (tolerates trailing inline comments)
    const declMatch = trimmed.match(/^(--[\w-]+):\s*([^;]+);/);
    if (!declMatch) continue;
    tokens[declMatch[1]] = declMatch[2].trim();
  }
  return tokens;
}

const rootBlock = css.slice(canonStart);
const lightBlock = rootBlock.slice(rootBlock.indexOf("{") + 1, rootBlock.indexOf("}"));
// selector must be followed by " {" so the comment prose mentioning
// '[data-theme="dark"]' is not mistaken for the actual dark block
const darkStart = css.indexOf('[data-theme="dark"] {', canonStart);
if (darkStart === -1) {
  console.error('Could not find [data-theme="dark"] block after canonical design tokens');
  process.exit(2);
}
const darkBlock = css.slice(darkStart + 1);

const lightTokens = parseThemeBlock(lightBlock);
const darkBody = darkBlock.slice(darkBlock.indexOf("{") + 1, darkBlock.indexOf("}"));
const darkTokens = parseThemeBlock(darkBody);

const canonicalColorFamilies = ["surface", "text", "border", "accent", "state", "elevation"];

for (const [token, lightValue] of Object.entries(LIGHT)) {
  const prop = `--${token}`;
  if (lightTokens[prop] !== lightValue) {
    fail(
      `canonical ${prop} (light) = "${lightTokens[prop] ?? "MISSING"}" ≠ snapshot "${lightValue}"`,
    );
  }
  const darkValue = DARK[token];
  if (darkTokens[prop] !== darkValue) {
    fail(`canonical ${prop} (dark) = "${darkTokens[prop] ?? "MISSING"}" ≠ snapshot "${darkValue}"`);
  }
}

for (const family of canonicalColorFamilies) {
  for (const prop of Object.keys(lightTokens)) {
    if (prop.startsWith(`--${family}-`) || prop === `--${family}`) {
      if (!(prop in darkTokens)) {
        fail(`canonical ${prop} is defined in :root but missing in [data-theme="dark"]`);
      }
    }
  }
  for (const prop of Object.keys(darkTokens)) {
    if (prop.startsWith(`--${family}-`) || prop === `--${family}`) {
      if (!(prop in lightTokens)) {
        fail(`canonical ${prop} is defined in [data-theme="dark"] but missing in :root`);
      }
    }
  }
}

/* ── 4. No --app-* color tokens anywhere (src + shipped html/css) ─────── */

function walk(dir, exts) {
  const out = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (entry === "node_modules" || entry === "dist") continue;
    if (statSync(full).isDirectory()) {
      out.push(...walk(full, exts));
    } else if (exts.includes(extname(full))) {
      out.push(full);
    }
  }
  return out;
}

const bannedRe = new RegExp(BANNED_APP_COLOR_TOKENS.join("|"), "g");
const shadcnVarRe = new RegExp(`var\\((${SHADCN_SEMANTIC_VARS.join("|")})\\)`, "g");

// src components + shipped static surfaces (public/, index.html, any css
// other than globals.css). globals.css is excluded from the shadcn-var scan
// because its compatibility layer legitimately references those aliases.
const targets = [
  ...walk(join(root, "src"), [".ts", ".tsx"]),
  ...walk(join(root, "public"), [".html", ".css"]),
];
for (const f of walk(root, [".css"])) {
  if (f !== cssPath) targets.push(f);
}
const indexHtml = join(root, "index.html");
if (statSync(indexHtml).isFile()) targets.push(indexHtml);

for (const file of targets) {
  const rel = file.slice(root.length + 1);
  const content = readFileSync(file, "utf-8");

  const banned = content.match(bannedRe);
  if (banned) {
    fail(`${rel}: removed --app-* color token(s) reintroduced: ${[...new Set(banned)].join(", ")}`);
  }

  const rawVars = content.match(shadcnVarRe);
  if (rawVars) {
    const unique = [...new Set(rawVars.map((v) => v.slice(5, -1)))];
    fail(
      `${rel}: raw shadcn semantic var(s) ${unique.join(", ")} — use canonical tokens (e.g. var(--accent)) or shadcn utility classes`,
    );
  }
}

/* ── Summary ─────────────────────────────────────────────────────────── */

if (drift > 0) {
  console.error(
    `\n${drift} token contract violation(s) detected. See docs/plans/active/ui-foundation-scale-hardening/PLAN.md P3.1.`,
  );
  process.exit(1);
}

console.log("Token contract: clean (canonical layer + shadcn aliases + component migration)");
process.exit(0);
