# Verification

## Automated

```text
cargo test -p db-pro-ui -- editor::renderer long_buffer_scrolls
```

Result (2026-09-18): renderer tests PASS including `long_buffer_scrolls_to_keep_end_caret_visible`.

## Runtime

pending: rebuild `db-pro-native`, open Query, wheel-scroll a long script, watch caret blink
