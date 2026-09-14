-- DB Pro v0.1 runtime SQLite fixture (pagination / filter / sort scale).
--
-- Deterministic, disposable, single-file: schema + seed data.
-- Build into a SCRATCH database only, never a user's real database:
--   sqlite3 /tmp/dbpro-v01-runtime/sqlite-runtime.db < fixtures/smoke/sqlite/runtime_fixture.sql
--
-- Sibling of `smoke_fixture.sql` (which stays small and shape-focused at 5 users).
-- This fixture exists to exercise row-grid pagination, filtering and sorting in the
-- packaged app, so it seeds a volume of rows rather than a breadth of column shapes.
--
-- Guaranteed contents (asserted by the companion verification queries):
--   Tables             : users, orders, order_items, user_roles (composite PK)
--   Views              : v_order_summary
--   Indexes            : 5 explicit (2 unique, 3 non-unique) + implicit PK/unique ones
--   Triggers           : trg_order_items_after_insert, trg_order_items_after_delete
--   CHECK constraints  : users.status, orders.status, orders.total_cents,
--                        order_items.quantity, order_items.unit_price_cents
--   FKs                : orders.user_id -> users.id
--                        order_items.order_id -> orders.id
--                        user_roles.user_id -> users.id
--   Rows               : users 60, orders 120, order_items 240, user_roles 120
--
-- Determinism: every value is derived arithmetically from a row number via a
-- recursive CTE. No random(), no datetime('now'), no wall-clock input — re-running
-- the script into a fresh file reproduces byte-identical data.

PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

DROP VIEW IF EXISTS v_order_summary;
DROP TRIGGER IF EXISTS trg_order_items_after_insert;
DROP TRIGGER IF EXISTS trg_order_items_after_delete;
DROP TABLE IF EXISTS order_items;
DROP TABLE IF EXISTS orders;
DROP TABLE IF EXISTS user_roles;
DROP TABLE IF EXISTS users;

-- ── users (PK, UNIQUE, CHECK) ─────────────────────────────────────
CREATE TABLE users (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    email       TEXT    NOT NULL,
    full_name   TEXT    NOT NULL,
    status      TEXT    NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active', 'invited', 'suspended')),
    created_at  TEXT    NOT NULL
);

CREATE UNIQUE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_status_created_at ON users (status, created_at);

-- ── user_roles (composite primary key) ────────────────────────────
CREATE TABLE user_roles (
    user_id     INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    role        TEXT    NOT NULL CHECK (role IN ('viewer', 'editor', 'admin')),
    granted_at  TEXT    NOT NULL,
    PRIMARY KEY (user_id, role)
) WITHOUT ROWID;

-- ── orders (FK to users, CHECK, non-unique sort index) ────────────
CREATE TABLE orders (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    reference   TEXT    NOT NULL,
    status      TEXT    NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'paid', 'shipped', 'cancelled')),
    total_cents INTEGER NOT NULL DEFAULT 0 CHECK (total_cents >= 0),
    placed_at   TEXT    NOT NULL
);

CREATE UNIQUE INDEX idx_orders_reference ON orders (reference);
CREATE INDEX idx_orders_user_placed_at ON orders (user_id, placed_at);
CREATE INDEX idx_orders_status_total_cents ON orders (status, total_cents);

-- ── order_items (FK to orders, CHECK, composite unique index) ─────
CREATE TABLE order_items (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id         INTEGER NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
    line_no          INTEGER NOT NULL CHECK (line_no > 0),
    sku              TEXT    NOT NULL,
    quantity         INTEGER NOT NULL CHECK (quantity > 0),
    unit_price_cents INTEGER NOT NULL CHECK (unit_price_cents > 0)
);

CREATE UNIQUE INDEX idx_order_items_order_sku ON order_items (order_id, sku);

-- ── Triggers: keep orders.total_cents derived from its items ──────
CREATE TRIGGER trg_order_items_after_insert
AFTER INSERT ON order_items
BEGIN
    UPDATE orders
       SET total_cents = total_cents + (NEW.quantity * NEW.unit_price_cents)
     WHERE id = NEW.order_id;
END;

CREATE TRIGGER trg_order_items_after_delete
AFTER DELETE ON order_items
BEGIN
    UPDATE orders
       SET total_cents = total_cents - (OLD.quantity * OLD.unit_price_cents)
     WHERE id = OLD.order_id;
END;

-- ── View: per-order rollup across all three tables ────────────────
CREATE VIEW v_order_summary AS
SELECT o.id            AS order_id,
       o.reference     AS reference,
       o.status        AS order_status,
       o.total_cents   AS total_cents,
       u.id            AS user_id,
       u.email         AS user_email,
       u.status        AS user_status,
       COUNT(i.id)     AS line_count,
       COALESCE(SUM(i.quantity), 0) AS unit_count
  FROM orders o
  JOIN users u       ON u.id = o.user_id
  LEFT JOIN order_items i ON i.order_id = o.id
 GROUP BY o.id, o.reference, o.status, o.total_cents, u.id, u.email, u.status;

-- ── Seed: 60 users ────────────────────────────────────────────────
WITH RECURSIVE seq(n) AS (
    SELECT 1 UNION ALL SELECT n + 1 FROM seq WHERE n < 60
)
INSERT INTO users (email, full_name, status, created_at)
SELECT printf('user%02d@fixture.dbpro.test', n),
       printf('Fixture User %02d', n),
       CASE n % 3 WHEN 0 THEN 'suspended' WHEN 1 THEN 'active' ELSE 'invited' END,
       datetime('2026-01-01 00:00:00', printf('+%d days', n))
  FROM seq;

-- ── Seed: 120 user_roles (2 roles per user, exercising the composite PK) ──
WITH RECURSIVE seq(n) AS (
    SELECT 1 UNION ALL SELECT n + 1 FROM seq WHERE n < 120
)
INSERT INTO user_roles (user_id, role, granted_at)
SELECT ((n - 1) % 60) + 1,
       CASE WHEN n <= 60 THEN 'viewer' ELSE 'editor' END,
       datetime('2026-01-01 00:00:00', printf('+%d days', ((n - 1) % 60) + 1))
  FROM seq;

-- ── Seed: 120 orders (2 per user) ─────────────────────────────────
-- total_cents starts at 0; the insert trigger accumulates it from order_items.
WITH RECURSIVE seq(n) AS (
    SELECT 1 UNION ALL SELECT n + 1 FROM seq WHERE n < 120
)
INSERT INTO orders (user_id, reference, status, total_cents, placed_at)
SELECT ((n - 1) % 60) + 1,
       printf('ORD-%05d', n),
       CASE n % 4 WHEN 0 THEN 'cancelled' WHEN 1 THEN 'pending' WHEN 2 THEN 'paid' ELSE 'shipped' END,
       0,
       datetime('2026-01-01 00:00:00', printf('+%d hours', n))
  FROM seq;

-- ── Seed: 240 order_items (2 lines per order) ─────────────────────
WITH RECURSIVE seq(n) AS (
    SELECT 1 UNION ALL SELECT n + 1 FROM seq WHERE n < 240
)
INSERT INTO order_items (order_id, line_no, sku, quantity, unit_price_cents)
SELECT ((n - 1) % 120) + 1,
       ((n - 1) / 120) + 1,
       printf('SKU-%04d', n),
       (n % 5) + 1,
       500 + ((n * 37) % 20000)
  FROM seq;

COMMIT;
