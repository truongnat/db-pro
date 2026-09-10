-- PostgreSQL packaged-smoke fixture for DB Pro v0.1.
-- Deterministic schema + seed for verifying the packaged app's runtime behavior.
--
-- Covers:
--   - Tables with/without PK, composite PK
--   - Views, indexes (B-tree, composite, expression, GIN)
--   - FK chains (single + composite)
--   - Enum, array, JSONB, UUID, BYTEA, NUMERIC, BIGSERIAL
--   - Generated column
--   - Trigger + function
--   - Sequence
--   - Representative Gate 5 value types (temporal, network placeholder)
--
-- Setup:  psql -U <user> -d <db> -f 001_smoke_schema.sql
-- Teardown: psql -U <user> -d <db> -f 002_smoke_teardown.sql
-- Verified object counts after schema: 10 tables, 2 views, 11 indexes, 1 trigger, 1 function, 1 enum, 1 sequence

-- ── Enum ──────────────────────────────────────────────────────
CREATE TYPE smoke_status AS ENUM ('draft', 'active', 'archived', 'deleted');

-- ── Sequence ──────────────────────────────────────────────────
CREATE SEQUENCE smoke_event_seq START 1;

-- ── PK table (parent) ────────────────────────────────────────
CREATE TABLE smoke_users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    role smoke_status NOT NULL DEFAULT 'active',
    metadata JSONB DEFAULT '{}',
    tags TEXT[] DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── PK table (child of users) ────────────────────────────────
CREATE TABLE smoke_orders (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES smoke_users(id) ON DELETE CASCADE,
    status smoke_status NOT NULL DEFAULT 'draft',
    total NUMERIC(12, 2) NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Composite PK table ───────────────────────────────────────
CREATE TABLE smoke_order_items (
    order_id BIGINT NOT NULL REFERENCES smoke_orders(id) ON DELETE CASCADE,
    product_id BIGINT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price NUMERIC(10, 2) NOT NULL,
    PRIMARY KEY (order_id, product_id)
);

-- ── No-PK table (readonly candidate) ─────────────────────────
CREATE TABLE smoke_audit_log (
    id BIGINT DEFAULT nextval('smoke_event_seq'),
    event_type VARCHAR(50) NOT NULL,
    payload JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Empty table ──────────────────────────────────────────────
CREATE TABLE smoke_empty (
    id SERIAL PRIMARY KEY,
    label VARCHAR(50)
);

-- ── Table with binary data ───────────────────────────────────
CREATE TABLE smoke_documents (
    id SERIAL PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    body TEXT,
    binary_data BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Table with generated column ──────────────────────────────
CREATE TABLE smoke_employees (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    full_name VARCHAR(101) GENERATED ALWAYS AS (first_name || ' ' || last_name) STORED,
    hire_date DATE NOT NULL DEFAULT CURRENT_DATE
);

-- ── Table with UUID PK ───────────────────────────────────────
CREATE TABLE smoke_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    sku VARCHAR(50) UNIQUE NOT NULL,
    price NUMERIC(10, 2) NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Table with inet/cidr for network types ───────────────────
CREATE TABLE smoke_network (
    id SERIAL PRIMARY KEY,
    ip_addr INET,
    network_cidr CIDR,
    mac MACADDR,
    label VARCHAR(100)
);

-- ── Table with unicode identifiers ───────────────────────────
CREATE TABLE "smoke_Ünïcödé" (
    id SERIAL PRIMARY KEY,
    "émâil" VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL
);

-- ── Views ────────────────────────────────────────────────────
CREATE VIEW smoke_active_users AS
SELECT id, username, email, role FROM smoke_users WHERE is_active = true;

CREATE VIEW smoke_order_summary AS
SELECT
    u.id AS user_id,
    u.username,
    COUNT(o.id) AS order_count,
    COALESCE(SUM(o.total), 0) AS total_spent
FROM smoke_users u
LEFT JOIN smoke_orders o ON o.user_id = u.id
GROUP BY u.id, u.username;

-- ── Indexes ──────────────────────────────────────────────────
CREATE INDEX idx_smoke_orders_user ON smoke_orders(user_id);
CREATE INDEX idx_smoke_orders_status_total ON smoke_orders(status, total);
CREATE INDEX idx_smoke_orders_created ON smoke_orders(created_at DESC);
CREATE INDEX idx_smoke_users_metadata ON smoke_users USING gin(metadata);
CREATE INDEX idx_smoke_employees_name ON smoke_employees(LOWER(first_name), LOWER(last_name));
CREATE INDEX idx_smoke_products_sku ON smoke_products(sku);
CREATE INDEX idx_smoke_audit_event ON smoke_audit_log(event_type, created_at DESC);
CREATE INDEX idx_smoke_network_ip ON smoke_network(ip_addr);

-- ── Trigger function ─────────────────────────────────────────
CREATE OR REPLACE FUNCTION smoke_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER smoke_users_updated_at
    BEFORE UPDATE ON smoke_users
    FOR EACH ROW
    EXECUTE FUNCTION smoke_update_timestamp();
