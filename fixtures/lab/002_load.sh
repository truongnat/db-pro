#!/bin/bash
# Loads fixtures/lab/seed-map.json, then expands it into lab.* tables.
set -euo pipefail

{
    printf '%s\n' 'INSERT INTO lab.seed_manifest (id, document) VALUES (1, $json$'
    cat /seed/seed-map.json
    printf '%s\n' '$json$::jsonb) ON CONFLICT (id) DO UPDATE SET document = EXCLUDED.document, loaded_at = now();'
} > /tmp/load_manifest.sql

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f /tmp/load_manifest.sql
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f /seed/003_expand.sql
