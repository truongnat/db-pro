-- PostgreSQL smoke fixture seed data.
-- Deterministic rows for packaged-app verification.
-- Run after 001_smoke_schema.sql.

-- ── Users (5 rows) ───────────────────────────────────────────
INSERT INTO smoke_users (username, email, is_active, role, metadata, tags) VALUES
    ('alice',   'alice@example.com',   true,  'active',   '{"plan": "pro", "login_count": 42}',  '{"beta","admin"}'),
    ('bob',     'bob@example.com',     true,  'active',   '{"plan": "free"}',                    '{"beta"}'),
    ('carol',   'carol@example.com',   false, 'archived', '{"plan": "pro", "reason": "inactive"}', '{}'),
    ('dave',    'dave@example.com',    true,  'active',   '{}',                                  '{"ops","admin"}'),
    ('ève',     'eve@example.com',     true,  'draft',    '{"plan": "free", "pending": true}',   '{}');

-- ── Orders (8 rows, FK → users) ──────────────────────────────
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
    ('user.created',    '{"user_id": 1}'),
    ('user.created',    '{"user_id": 2}'),
    ('order.created',   '{"order_id": 1}'),
    ('order.shipped',   '{"order_id": 6}'),
    ('user.deactivated', '{"user_id": 3}');

-- ── Documents (BYTEA, 3 rows) ────────────────────────────────
INSERT INTO smoke_documents (title, body, binary_data) VALUES
    ('README',      'Smoke test document body.', X'48656C6C6F'),
    ('Empty Doc',   '',                         NULL),
    ('Binary Only', NULL,                       X'DEADBEEF');

-- ── Employees (generated column, 3 rows) ─────────────────────
INSERT INTO smoke_employees (first_name, last_name, hire_date) VALUES
    ('John',  'Doe',    '2024-01-15'),
    ('Jane',  'Smith',  '2024-03-20'),
    ('Bob',   'Johnson', '2023-11-01');

-- ── Products (UUID PK, 5 rows) ───────────────────────────────
INSERT INTO smoke_products (name, sku, price, metadata) VALUES
    ('Widget A',  'SKU-001', 150.00, '{"color": "red"}'),
    ('Widget B',  'SKU-002', 200.00, '{"color": "blue"}'),
    ('Gadget C',  'SKU-003', 42.00,  '{"weight_kg": 0.5}'),
    ('Gadget D',  'SKU-004', 25.00,  '{}'),
    ('Tool E',    'SKU-005', 10.00,  '{"discontinued": true}');

-- ── Network (inet/cidr/macaddr, 3 rows) ──────────────────────
INSERT INTO smoke_network (ip_addr, network_cidr, mac, label) VALUES
    ('192.168.1.1',   '192.168.1.0/24',  '08:00:2b:01:02:03', 'office-router'),
    ('10.0.0.5',      '10.0.0.0/8',      '00:1a:2b:3c:4d:5e', 'vpn-server'),
    ('172.16.0.100',  NULL,               NULL,                'dynamic-host');

-- ── Unicode table (2 rows) ───────────────────────────────────
INSERT INTO "smoke_Ünïcödé" ("émâil", name) VALUES
    ('ünïcödé@example.com',  'Test Ünïcödé'),
    ('müller@example.com',   'Müller');
