# Findings

## Confirmed P2 — SQLite actor has zero lock wait and uncaught handler panics

- Issues: #493 and #494.
- `SqliteActor::spawn` opened a connection without setting `busy_timeout`; SQLite therefore returned `SQLITE_BUSY` immediately on lock contention.
- The worker called handlers directly. An unwinding panic dropped the actor thread and responders, leaving the connection apparently alive to callers while future requests failed through closed channels.
- Fix: set a 5-second native SQLite busy timeout and wrap every command handler with `catch_unwind(AssertUnwindSafe(...))`, returning structured errors while retaining the worker loop.
