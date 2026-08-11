#!/usr/bin/env node
/* eslint-disable no-console */

/**
 * Token drift detector — ensures the shadcn compatibility layer in globals.css
 * does not reintroduce raw color values. All shadcn tokens should alias --app-* tokens.
 *
 * Allowed raw values in the shadcn layer:
 *   --radius, --chart-*, *-foreground: #ffffff, --info (light only), --overlay
 *
 * Exit 0 = clean, Exit 1 = drift detected.
 */

import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const cssPath = resolve(__dirname, "../src/styles/globals.css");
const css = readFileSync(cssPath, "utf-8");

const ALLOWED_PROP_PREFIXES = ["--radius", "--chart-"];
const ALLOWED_PROP_SUFFIXES = ["-foreground"];
const ALLOWED_PROPS_EXACT = new Set(["--info", "--overlay"]);

const RAW_COLOR_RE = /#[0-9a-fA-F]{3,8}\b|rgba?\(|oklch\(|hsla?\(/;

const compatStart = css.indexOf("shadcn compatibility layer");
const compatEnd = css.indexOf("App-level design tokens");

if (compatStart === -1 || compatEnd === -1) {
  console.error("Could not find shadcn compatibility layer boundaries in globals.css");
  process.exit(2);
}

const compatBlock = css.slice(compatStart, compatEnd);
const lines = compatBlock.split("\n");

let drift = 0;

for (let i = 0; i < lines.length; i++) {
  const line = lines[i].trim();
  if (!line || line.startsWith("/*") || line.startsWith("*") || line.startsWith("//")) continue;

  const declMatch = line.match(/^(--[\w-]+):\s*(.+);$/);
  if (!declMatch) continue;

  const [, prop, value] = declMatch;

  if (ALLOWED_PROP_PREFIXES.some((p) => prop.startsWith(p))) continue;
  if (ALLOWED_PROP_SUFFIXES.some((s) => prop.endsWith(s))) continue;
  if (ALLOWED_PROPS_EXACT.has(prop)) continue;
  if (value.startsWith("var(")) continue;

  if (RAW_COLOR_RE.test(value)) {
    const lineNum = css.slice(0, css.indexOf(line)).split("\n").length;
    console.error(`DRIFT: ${prop} has raw value "${value}" (line ~${lineNum})`);
    console.error(`  → Should alias to var(--app-*) instead`);
    drift++;
  }
}

if (drift > 0) {
  console.error(`\n${drift} token drift(s) detected. Run: review globals.css shadcn compatibility layer.`);
  process.exit(1);
}

console.log("Token contract: clean (no raw color drift in shadcn compatibility layer)");
process.exit(0);
