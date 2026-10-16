# Checklist

- [x] Confirm actor spawn omitted busy timeout and command handlers could unwind the worker.
- [x] Configure per-connection busy timeout.
- [x] Recover per-command panics and report transaction outcome as unknown.
- [x] Add regression tests for timeout, worker survival, and transaction panic reporting.
- [x] Run focused tests, infrastructure/core check, format, and clean-code scan.
- [ ] Rebase, commit, push, and open PR linking #493 and #494.
