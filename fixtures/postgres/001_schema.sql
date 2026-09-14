-- PostgreSQL fixture schema for DB Pro integration tests.
-- Covers: normal tables, empty tables, no PK, composite PK, FKs, nullable,
-- unicode, quoted identifiers, large text, JSONB, UUID, generated columns,
-- enum, array, expression index, views, triggers, functions, sequences.

-- ── Enum type ──────────────────────────────────────────────────
CREATE TYPE order_status AS ENUM ('pending', 'processing', 'shipped', 'delivered', 'cancelled');

-- ── Sequence ───────────────────────────────────────────────────
CREATE SEQUENCE audit_seq START 1;

-- ── Categories ─────────────────────────────────────────────────
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT
);

-- ── Products ───────────────────────────────────────────────────
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    sku VARCHAR(50) UNIQUE NOT NULL,
    price NUMERIC(10, 2) NOT NULL DEFAULT 0,
    category_id INTEGER REFERENCES categories(id),
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- ── Users ──────────────────────────────────────────────────────
CREATE TABLE "Ünïcödé Üsers" (
    id SERIAL PRIMARY KEY,
    "émâil" VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- ── Orders ─────────────────────────────────────────────────────
CREATE TABLE orders (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    status order_status NOT NULL DEFAULT 'pending',
    total NUMERIC(12, 2) NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- ── Order items (composite PK) ─────────────────────────────────
CREATE TABLE order_items (
    order_id BIGINT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price NUMERIC(10, 2) NOT NULL,
    PRIMARY KEY (order_id, product_id)
);

-- ── Table without PK ───────────────────────────────────────────
CREATE TABLE audit_logs (
    id BIGINT DEFAULT nextval('audit_seq'),
    event_type VARCHAR(50) NOT NULL,
    payload JSONB,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- ── Empty table ────────────────────────────────────────────────
CREATE TABLE empty_table (
    id SERIAL PRIMARY KEY,
    label VARCHAR(50)
);

-- ── Table with weird identifiers ───────────────────────────────
CREATE TABLE "weird""name" (
    id SERIAL PRIMARY KEY,
    "col with spaces" VARCHAR(50),
    "SELECT" VARCHAR(50)  -- reserved word as column name
);

-- ── Large text table ───────────────────────────────────────────
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    title VARCHAR(200),
    body TEXT,
    binary_data BYTEA
);

-- ── Generated column ───────────────────────────────────────────
CREATE TABLE employees (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    full_name VARCHAR(101) GENERATED ALWAYS AS (first_name || ' ' || last_name) STORED,
    hire_date DATE NOT NULL DEFAULT CURRENT_DATE
);

-- ── Views ──────────────────────────────────────────────────────
CREATE VIEW active_users AS
SELECT id, "émâil", name FROM "Ünïcödé Üsers" WHERE is_active = true;

CREATE VIEW order_summary AS
SELECT
    o.id AS order_id,
    o.status,
    o.total,
    COUNT(oi.product_id) AS item_count
FROM orders o
LEFT JOIN order_items oi ON oi.order_id = o.id
GROUP BY o.id, o.status, o.total;

-- ── Indexes ────────────────────────────────────────────────────
CREATE INDEX idx_products_category ON products(category_id);
CREATE INDEX idx_products_metadata ON products USING gin(metadata);
CREATE INDEX idx_orders_user ON orders(user_id);
CREATE INDEX idx_orders_created ON orders(created_at DESC);
CREATE INDEX idx_orders_status_total ON orders(status, total);
-- Expression index
CREATE INDEX idx_employees_lower_name ON employees(LOWER(first_name), LOWER(last_name));

-- ── Trigger function ───────────────────────────────────────────
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER products_updated_at
    BEFORE UPDATE ON products
    FOR EACH ROW
    EXECUTE FUNCTION update_timestamp();

-- ── Standalone function ────────────────────────────────────────
CREATE OR REPLACE FUNCTION calculate_total(p_order_id BIGINT)
RETURNS NUMERIC AS $$
BEGIN
    RETURN (SELECT COALESCE(SUM(quantity * unit_price), 0)
            FROM order_items WHERE order_id = p_order_id);
END;
$$ LANGUAGE plpgsql;

-- ── Decoder matrix (Gate 5 B5, #59) ────────────────────────────
-- One column per value class the provider contract defines, so the decoder is
-- exercised against a real server instead of unit-level metadata: temporal
-- classes, exact numerics, structured classes, and the provider-specific
-- enum/domain/array/range/composite classes the fallback has to survive.
CREATE DOMAIN positive_quantity AS INTEGER CHECK (VALUE > 0);
CREATE DOMAIN postal_code AS TEXT;
CREATE TYPE decoder_pair AS (left_value INTEGER, right_value TEXT);

CREATE TABLE decoder_matrix (
    id SMALLINT PRIMARY KEY,             -- int2
    flag BOOLEAN NOT NULL,               -- bool
    count INTEGER,                       -- int4
    big_count BIGINT,                    -- int8
    ratio REAL,                          -- float4
    precise_ratio DOUBLE PRECISION,      -- float8
    amount NUMERIC(24, 4),               -- numeric/decimal, high precision
    calendar_date DATE,                  -- date
    wall_time TIME(6),                   -- time
    zoned_time TIMETZ,                   -- timetz
    local_stamp TIMESTAMP(6),            -- timestamp
    instant TIMESTAMPTZ,                 -- timestamptz
    span INTERVAL,                       -- interval
    token UUID,                          -- uuid
    doc JSON,                            -- json
    payload JSONB,                       -- jsonb
    blob BYTEA,                          -- bytea
    address INET,                        -- inet
    network CIDR,                        -- cidr
    status order_status,                 -- enum
    quantity positive_quantity,          -- domain over int4
    postal postal_code,                  -- domain over text
    labels TEXT[],                       -- array
    slot INT4RANGE,                      -- range (no explicit decoder arm)
    pair decoder_pair,                   -- composite (no flat representation)
    missing TEXT                         -- nulls
);
