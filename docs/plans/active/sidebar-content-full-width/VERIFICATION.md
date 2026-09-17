# Verification — sidebar-content-full-width

## Automated

```text
cargo check -p db-pro-ui --all-targets
→ PASS (2026-09-17)
```

## Runtime (pending)

Rebuild `db-pro-native`, open Explorer with a failed connection:

1. Tree hover wash extends to near the blue drag line (not mid-panel).
2. Long error hint truncates near the drag edge, not ~halfway across the sidebar.
3. `New query` / filter and tree share the same content width.
4. Dragging the separator still respects 220–380.

## Providers

n/a — layout-only UI change (PostgreSQL / SQLite unaffected).
