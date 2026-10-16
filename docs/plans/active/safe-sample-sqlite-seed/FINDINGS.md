# Findings — Safe SQLite Sample Seeding

## Root Cause

Startup used a fixed `/tmp/db_pro_sample.db`, followed any existing path (including symlinks), and registered it as `Sample E-Commerce (SQLite)` with `readonly = false`. Fixture initialization errors were ignored.

## Fix Strategy

Use `runtime.data_dir()/sample-ecommerce.sqlite`. Atomically reserve a new file with `create_new`; initialize it with a DB Pro SQLite application id and marker. If the path already exists, reject symlinks and only reuse a regular SQLite file whose marker and application id match. Do not register the connection if validation fails. Mark it read-only and migrate a prior built-in sample entry away from the legacy `/tmp` path.
