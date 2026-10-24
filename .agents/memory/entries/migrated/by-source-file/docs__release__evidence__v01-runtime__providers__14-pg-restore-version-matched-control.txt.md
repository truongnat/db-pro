---
schema_version: 1
id: "MEM-SRC-9158932b4749ed70"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/14-pg-restore-version-matched-control.txt"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/14-pg-restore-version-matched-control.txt"
source_sha256: "9158932b4749ed70d2544955c1dcfe48b3caf4ef3c7616fc26fa822a6dc65841"
mirror_body_sha256: "9158932b4749ed70d2544955c1dcfe48b3caf4ef3c7616fc26fa822a6dc65841"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
### PostgreSQL backup/restore — version-matched control (corrected: -U dbpro)

$ dump AND restore both with the version-matched 16.15 tools inside the container
pg_dump (PostgreSQL) 16.15 (Debian 16.15-1.pgdg13+2)
  pg_dump 16.15 exit=0
    ;     Dump Version: 1.15-0
    ;     Format: CUSTOM
    ;     Dumped by pg_dump version: 16.15 (Debian 16.15-1.pgdg13+2)
  pg_restore 16.15 exit=0

  restored row counts (version-matched control):
    orders=4 order_items=6
  fingerprint (version-matched control) — compare with source 3146f5fe4be0c45ed998cff6f386364d:
    orders_fp=3146f5fe4be0c45ed998cff6f386364d
  object counts:
    S=8
    i=17
    r=10
    v=2
  003_verify.sql against the version-matched restore:
    PASS (exit 0)
