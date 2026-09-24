# Verification

Baseline: `d52e752fd1f7becdd5abbcdf6a12662959c08a72`.

## Commands

```text
cargo check -p db-pro-ui --offline
  exit 0

cargo test -p db-pro-ui --offline --lib welcome_top_inset
  1 passed / 0 failed / 0 ignored

cargo test -p db-pro-ui --offline --lib the_filter_field_border
  1 passed / 0 failed / 0 ignored

cargo fmt -p db-pro-ui -- --check
  exit 0
```

Not run: `cargo clippy --workspace`, `cargo test --workspace`, `cargo build --release --locked -p db-pro-native` without the capture feature. The capture binary was built:

```text
cargo build --release --locked --offline -p db-pro-native --features capture
  Finished release in 48.90s, exit 0
```

## UI captures

Empty catalog, keyring disabled. Framebuffer is slightly inside the requested window (window chrome).

| Surface | Requested | File |
|---|---|---|
| Welcome | 1280×800 | `/opt/cursor/artifacts/welcome_1280x800.png` (1267×792) |
| Welcome | 1440×900 | `/opt/cursor/artifacts/welcome_1440x900.png` (1425×891) |
| Welcome | 1920×1080 | `/opt/cursor/artifacts/welcome_1920x1080.png` (1900×1069) |
| Query | 1280×800 | `/opt/cursor/artifacts/query_1280x800.png` (1267×792) |
| Query | 1440×900 | `/opt/cursor/artifacts/query_1440x900.png` (1425×891) |
| Query | 1920×1080 | `/opt/cursor/artifacts/query_1920x1080.png` (1900×1069) |

Accent pixels for the Run control on the 1440 query capture sit on rows 72–95, in the toolbar under the tab strip, not on the status bar.

## Provider matrix

| Provider | Automated | Live | Notes |
|---|---|---|---|
| PostgreSQL | N/A | N/A | No SQL or connection behavior changed |
| SQLite | N/A | N/A | No SQL or connection behavior changed |

## Lab database

`docker-compose.lab.yml` on host port 55433, database `dbpro_lab`, user `dbpro`. Seed mapping: `fixtures/lab/seed-map.json`, expanded by `003_expand.sql`.

```text
sudo docker compose -f docker-compose.lab.yml up -d
PGPASSWORD=dbpro_test psql -h 127.0.0.1 -p 55433 -U dbpro -d dbpro_lab
```

Measured 2026-09-24 after init (`INSERT 0 1` then `DO` in the container log):

| Table | Rows |
|---|---|
| lab.products | 1200 |
| lab.customers | 8000 |
| lab.orders | 20000 |
| lab.order_items | 40000 |
| lab.events | 12000 |

`SELECT status, count(*) FROM lab.orders GROUP BY status` returned six statuses, 3333–3334 each. The Welcome starter joining customers to orders returned rows (top spend 6309.90).

Welcome capture after the composer redesign: `/opt/cursor/artifacts/welcome_lab_1440.png`.

## Remaining

- P0: 0
- P1: 0
- P2: workspace clippy and the full test suite were not run in this pass
- Independent review is still required; this plan stays under `docs/plans/active/`
