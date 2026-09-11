/*
 * bench/run-bench.js — headless-Chrome CDP benchmark runner.
 *
 * Usage: node run-bench.js <file-url> [timeout-ms]
 *
 * Launches Chrome headless, navigates to the harness page, polls document.title
 * until "BENCH_READY", then extracts window.__bench and console errors.
 * Prints a JSON envelope on stdout.
 *
 * Requires Node >= 22 (global fetch + WebSocket).
 */
"use strict";

const { spawn } = require("child_process");
const fs = require("fs");

const SCREENSHOT = process.env.SCREENSHOT || null;

const URL = process.argv[2];
const TIMEOUT_MS = parseInt(process.argv[3] || "240000", 10);
const PORT = 9333 + Math.floor(Math.random() * 500);

const chrome = spawn(
  "google-chrome",
  [
    "--headless=new",
    "--disable-gpu",
    "--no-sandbox",
    "--disable-dev-shm-usage",
    "--window-size=1600,1000",
    `--remote-debugging-port=${PORT}`,
    "--user-data-dir=/tmp/bench-chrome-profile-" + PORT,
    "about:blank",
  ],
  { stdio: ["ignore", "ignore", "pipe"] }
);

let ws = null;
const pending = new Map();
const consoleErrors = [];
let msgId = 0;

function send(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = ++msgId;
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params }));
  });
}

function handleMessage(msg) {
  if (msg.id && pending.has(msg.id)) {
    const { resolve, reject } = pending.get(msg.id);
    pending.delete(msg.id);
    if (msg.error) reject(new Error(msg.error.message));
    else resolve(msg.result);
    return;
  }
  if (msg.method === "Runtime.exceptionThrown") {
    const d = msg.params.exceptionDetails;
    const desc = (d.exception && d.exception.description) || d.text || "exception";
    consoleErrors.push("EXCEPTION: " + desc.split("\n")[0]);
  }
  if (msg.method === "Runtime.consoleAPICalled" && msg.params.type === "error") {
    const text = msg.params.args.map((a) => a.value ?? a.description ?? "").join(" ");
    consoleErrors.push("CONSOLE.ERROR: " + text);
  }
}

async function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function waitForDebugger() {
  const deadline = Date.now() + 15000;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`http://127.0.0.1:${PORT}/json/list`);
      if (res.ok) return;
    } catch (_) {
      /* retry */
    }
    await sleep(300);
  }
  throw new Error("CDP endpoint never came up");
}

async function evalJs(expression) {
  const res = await send("Runtime.evaluate", {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  if (res.exceptionDetails) {
    throw new Error("eval failed: " + JSON.stringify(res.exceptionDetails.exception?.description || res.exceptionDetails.text));
  }
  return res.result?.value;
}

async function main() {
  await waitForDebugger();
  const targets = await (await fetch(`http://127.0.0.1:${PORT}/json/list`)).json();
  const page = targets.find((t) => t.type === "page");
  if (!page) throw new Error("no page target");

  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res, rej) => {
    ws.onopen = res;
    ws.onerror = () => rej(new Error("ws error"));
  });
  ws.onmessage = (ev) => handleMessage(JSON.parse(ev.data));

  await send("Runtime.enable");
  await send("Page.enable");

  const tStart = Date.now();
  await send("Page.navigate", { url: URL });

  // Poll for BENCH_READY
  let benchJson = null;
  for (;;) {
    await sleep(2000);
    const title = await evalJs("document.title");
    if (typeof title === "string" && title.startsWith("BENCH_READY")) break;
    if (Date.now() - tStart > TIMEOUT_MS) break;
  }

  const elapsedMs = Date.now() - tStart;
  benchJson = await evalJs("typeof window.__bench !== 'undefined' ? JSON.stringify(window.__bench) : null");
  const phase = await evalJs("document.getElementById && document.getElementById('phase') ? document.getElementById('phase').textContent : 'n/a'");

  if (SCREENSHOT) {
    try {
      const shot = await send("Page.captureScreenshot", { format: "png" });
      fs.writeFileSync(SCREENSHOT, Buffer.from(shot.data, "base64"));
    } catch (e) {
      consoleErrors.push("screenshot failed: " + e.message);
    }
  }

  const out = {
    url: URL,
    finished: benchJson !== null,
    elapsedMs,
    phase,
    bench: benchJson ? JSON.parse(benchJson) : null,
    consoleErrors: consoleErrors.slice(0, 12),
  };
  console.log(JSON.stringify(out, null, 1));
}

main()
  .catch((e) => {
    console.log(JSON.stringify({ url: URL, finished: false, error: String(e && e.message || e), consoleErrors }, null, 1));
  })
  .finally(() => {
    setTimeout(() => {
      try { ws && ws.close(); } catch (_) {}
      chrome.kill("SIGKILL");
    }, 200);
  });
