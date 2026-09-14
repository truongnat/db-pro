# Screenshots — none captured, and none can be from this environment

**This directory is intentionally empty.** No screenshot was taken during the `v01-runtime`
session, no screenshot is claimed, and no visual verdict of any kind is asserted anywhere in the
`v01-runtime` evidence tree.

The reason is that this shell has **no window-server access**, so it cannot observe a window at
all. GUI automation is unavailable, re-verified live during this session — raw output in
`../providers/21-gui-unavailability-probes.txt`. The four blockers, verbatim:

1. **Orca computer-use helper — `runtime_unavailable`.** Both
   `orca computer get-app-state --app com.dbpro.app --json` and
   `orca computer list-windows --app com.dbpro.app --json` return:

   ```json
   {
     "ok": false,
     "error": {
       "code": "runtime_unavailable",
       "message": "Could not read Orca runtime metadata at /Users/truongdev/Library/Application Support/orca/orca-runtime.json. Start the Orca app first."
     }
   }
   ```

2. **AppleScript / System Events — TCC denial.**
   `osascript -e 'tell application "System Events" to get name of first process'` returns:

   ```
   40:44: execution error: Not authorized to send Apple events to System Events. (-1743)
   ```

3. **`screencapture` — no display reachable.** `screencapture -x <path>` returns:

   ```
   could not create image from display
   ```

   and creates no file.

4. **No accessibility tree.** The three mechanisms above are the ways to read one; all fail before
   returning a tree, so no UI element can be enumerated, queried, or clicked.

These are host and authorization limits, not application failures. Precisely because they could not
be lifted, **no app-side GUI claim can be made either way** — neither that the UI works nor that it
does not.

## What this means for the release record

Every GUI-dependent item remains `PENDING_HUMAN`, owned by the coordinator. The human runbook that
would close it is `docs/release/evidence/v01-06/14-install-smoke.txt` §8. The code-derived surface
description `docs/release/0.1.0-ui-visual-description.md` is **not** a screenshot and must not be
cited as one.

## What a human should capture here

When a desktop session is available, this directory is where the captures belong — at minimum:
the window rendering; Settings navigation including the light/dark switch; a SQLite connection
created through the UI; `SELECT 1;` and `SELECT * FROM items;` results; row-grid
pagination/filter/sort against `fixtures/smoke/sqlite/runtime_fixture.sql`; the capability-gated
"Running…" state while a query runs; and a clean window close / ⌘Q exit. Record the display
resolution and the exact build SHA alongside each capture.
