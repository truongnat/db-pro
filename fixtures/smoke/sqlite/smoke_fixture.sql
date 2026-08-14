-- SQLite packaged-smoke fixture for DB Pro v0.1.
-- Single-file fixture: schema + seed data.
--
-- Covers:
--   - Tables with/without PK, composite PK
--   - Views, indexes (B-tree, composite)
--   - FK chains (single + composite)
--   - JSON stored as text, BLOB, NUMERIC (REAL)
--   - Trigger
--   - CHECK constraint (adaptable to #68 keep/defer)
--   - Unicode identifiers
--   - Representative storage classes (INTEGER, REAL, TEXT, BLOB, NULL)
--
-- Setup:    sqlite3 smoke.db < smoke_fixture.sql
-- Teardown: rm smoke.db (or re-run from clean)
-- Verified object counts: 10 tables, 2 views, 6 indexes, 1 trigger

-- ── PK table (parent) ────────────────────────────────────────
CREATE TABLE smoke_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    is_active INTEGER NOT NULL DEFAULT 1,
    role TEXT NOT NULL DEFAULT 'active' CHECK(role IN ('draft','active','archived','deleted')),
    metadata TEXT DEFAULT '{}',
    tags TEXT DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── PK table (child of users) ────────────────────────────────
CREATE TABLE smoke_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES smoke_users(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'draft',
    total REAL NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Composite PK table ───────────────────────────────────────
CREATE TABLE smoke_order_items (
    order_id INTEGER NOT NULL REFERENCES smoke_orders(id),
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price REAL NOT NULL,
    PRIMARY KEY (order_id, product_id)
);

-- ── No-PK table (readonly candidate) ─────────────────────────
CREATE TABLE smoke_audit_log (
    event_type TEXT NOT NULL,
    payload TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Empty table ──────────────────────────────────────────────
CREATE TABLE smoke_empty (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label TEXT
);

-- ── Table with BLOB data ─────────────────────────────────────
CREATE TABLE smoke_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    body TEXT,
    binary_data BLOB,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Employees (no generated columns in SQLite) ───────────────
CREATE TABLE smoke_employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    hire_date TEXT NOT NULL DEFAULT (date('now'))
);

-- ── Products (TEXT PK simulating UUID) ───────────────────────
CREATE TABLE smoke_products (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sku TEXT NOT NULL UNIQUE,
    price REAL NOT NULL DEFAULT 0,
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── Network types stored as text (SQLite has no native inet) ─
CREATE TABLE smoke_network (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ip_addr TEXT,
    network_cidr TEXT,
    mac TEXT,
    label TEXT
);

-- ── Unicode identifiers ──────────────────────────────────────
CREATE TABLE "smoke_Ünïcödé" (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    "émâil" TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);

-- ── Views ────────────────────────────────────────────────────
CREATE VIEW smoke_active_users AS
SELECT id, username, email, role FROM smoke_users WHERE is_active = 1;

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
CREATE INDEX idx_smoke_products_sku ON smoke_products(sku);
CREATE INDEX idx_smoke_audit_event ON smoke_audit_log(event_type, created_at DESC);
CREATE INDEX idx_smoke_network_ip ON smoke_network(ip_addr);

-- ── Trigger ──────────────────────────────────────────────────
CREATE TRIGGER smoke_users_updated_at
    AFTER UPDATE ON smoke_users
    FOR EACH ROW
BEGIN
    UPDATE smoke_users SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- ══════════════════════════════════════════════════════════════
-- SEED DATA
-- ══════════════════════════════════════════════════════════════

-- ── Users (5 rows) ───────────────────────────────────────────
INSERT INTO smoke_users (username, email, is_active, role, metadata, tags) VALUES
    ('alice',   'alice@example.com',   1, 'active',   '{"plan":"pro","login_count":42}',   '["beta","admin"]'),
    ('bob',     'bob@example.com',     1, 'active',   '{"plan":"free"}',                   '["beta"]'),
    ('carol',   'carol@example.com',   0, 'archived', '{"plan":"pro","reason":"inactive"}', '[]'),
    ('dave',    'dave@example.com',    1, 'active',   '{}',                                '["ops","admin"]'),
    ('eve',     'eve@example.com',     1, 'draft',    '{"plan":"free","pending":true}',     '[]');

-- ── Orders (8 rows) ──────────────────────────────────────────
INSERT INTO smoke_orders (user_id, status, total, notes) VALUES
    (1, 'active',   150.00, 'First order'),
    (1, 'active',   275.50, NULL),
    (1, 'archived', 42.00,  'Returned'),
    (2, 'active',   89.99,  NULL),
    (2, 'draft',    0.00,   'Empty cart'),
    (4, 'active',   500.00, 'Bulk order'),
    (4, 'deleted',  25.00,  'Cancelled'),
    (5, 'draft',    10.00,  NULL);

-- ── Order items (composite PK, 10 rows) ──────────────────────
INSERT INTO smoke_order_items (order_id, product_id, quantity, unit_price) VALUES
    (1, 1, 1, 150.00),
    (2, 1, 1, 200.00),
    (2, 2, 1, 75.50),
    (3, 3, 1, 42.00),
    (4, 2, 1, 89.99),
    (6, 1, 2, 200.00),
    (6, 3, 1, 100.00),
    (7, 4, 1, 25.00),
    (8, 5, 1, 10.00),
    (1, 3, 2, 0.00);

-- ── Audit log (no PK, 5 rows) ────────────────────────────────
INSERT INTO smoke_audit_log (event_type, payload) VALUES
    ('user.created',     '{"user_id":1}'),
    ('user.created',     '{"user_id":2}'),
    ('order.created',    '{"order_id":1}'),
    ('order.shipped',    '{"order_id":6}'),
    ('user.deactivated', '{"user_id":3}');

-- ── Documents (BLOB, 3 rows) ─────────────────────────────────
INSERT INTO smoke_documents (title, body, binary_data) VALUES
    ('README',      'Smoke test document body.', X'48656C6C6F'),
    ('Empty Doc',   '',                         NULL),
    ('Binary Only', NULL,                       X'DEADBEEF');

-- ── Employees (3 rows) ───────────────────────────────────────
INSERT INTO smoke_employees (first_name, last_name, hire_date) VALUES
    ('John',  'Doe',     '2024-01-15'),
    ('Jane',  'Smith',   '2024-03-20'),
    ('Bob',   'Johnson', '2023-11-01');

-- ── Products (TEXT UUID PK, 5 rows) ──────────────────────────
INSERT INTO smoke_products (id, name, sku, price, metadata) VALUES
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'Widget A',  'SKU-001', 150.00, '{"color":"red"}'),
    ('b1ffcd00-ad1c-5fg9-cc7e-7cc0ce491b22', 'Widget B',  'SKU-002', 200.00, '{"color":"blue"}'),
    ('c2ggde11-be2d-6gh0-dd8f-8dd1df502c33', 'Gadget C',  'SKU-003', 42.00,  '{"weight_kg":0.5}'),
    ('d3hhef22-cf3e-7hi1-ee9g-9ee2eg613d44', 'Gadget D',  'SKU-004', 25.00,  '{}'),
    ('e4iifg33-dg4f-8ij2-ff0h-0ff3fh724e55', 'Tool E',    'SKU-005', 10.00,  '{"discontinued":true}');

-- ── Network (3 rows) ─────────────────────────────────────────
INSERT INTO smoke_network (ip_addr, network_cidr, mac, label) VALUES
    ('192.168.1.1',   '192.168.1.0/24',  '08:00:2b:01:02:03', 'office-router'),
    ('10.0.0.5',      '10.0.0.0/8',      '00:1a:2b:3c:4d:5e', 'vpn-server'),
    ('172.16.0.100',  NULL,               NULL,                'dynamic-host');

-- ── Unicode table (2 rows) ───────────────────────────────────
INSERT INTO "smoke_Ünïcödé" ("émâil", name) VALUES
    ('unicode@example.com',  'Test Unicode'),
    ('muller@example.com',   'Muller');
