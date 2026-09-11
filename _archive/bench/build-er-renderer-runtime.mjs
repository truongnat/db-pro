/*
 * bench/build-er-renderer-runtime.mjs — bundles the REAL app renderer source
 * into the standalone runtime harness (er-renderer-runtime.html).
 *
 * Usage: node build-er-renderer-runtime.mjs
 *
 * The harness must exercise the actual `CytoscapeErRenderer` + the actual
 * approximate-layout code — not a reimplementation — so we bundle them from
 * the app source with esbuild (cytoscape is bundled too; the harness imports
 * window.ErRuntime). Run this before `node run-bench.js er-renderer-runtime.html`.
 */
import { createRequire } from "node:module";
import { rmSync, statSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = resolve(fileURLToPath(new URL(".", import.meta.url)));
const root = resolve(__dirname, "..");
const entry = join(root, "frontend/src/modules/er-diagram/renderer/cytoscape-renderer.ts");
const approx = join(root, "frontend/src/modules/er-diagram/utils/approximate-layout.ts");
const outfile = join(__dirname, "er-renderer-runtime.bundle.js");
const tmpEntry = join(__dirname, ".runtime-entry.mjs");

// esbuild lives in frontend/node_modules (sibling of bench/) — ESM bare-import
// resolution only walks ancestor directories, so anchor require() there.
const frontendRequire = createRequire(join(root, "frontend/package.json"));
const { build } = frontendRequire("esbuild");

async function main() {
  const start = Date.now();
  writeFileSync(
    tmpEntry,
    `export { CytoscapeErRenderer } from ${JSON.stringify(entry)};\n` +
      `export { computeApproximateOverviewLayout } from ${JSON.stringify(approx)};\n` +
      `export { refinePositions, meanEdgeLength, computeOptimalDistance, PROGRESSIVE_MIN_NODES } from ${JSON.stringify(
        join(root, "frontend/src/modules/er-diagram/utils/force-refine.ts"),
      )};\n`,
  );

  await build({
    entryPoints: [tmpEntry],
    bundle: true,
    format: "iife",
    globalName: "ErRuntime",
    outfile,
    minify: false,
    sourcemap: false,
    target: ["chrome110"],
    logLevel: "warning",
  });
  // The temp entry + bundle are build artifacts (gitignored); keep the tree clean.
  rmSync(tmpEntry, { force: true });
  console.log(
    `bundled real renderer → ${outfile} (${(statSync(outfile).size / 1024).toFixed(0)} kB, ${Date.now() - start} ms)`,
  );
}

main().catch((e) => {
  console.error("build failed:", e.message);
  process.exit(1);
});
