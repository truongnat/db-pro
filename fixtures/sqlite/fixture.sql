-- SQLite fixture schema + seed data for DB Pro integration tests.
-- Single-file fixture covering schema and data.

-- ── Categories ──────────────────────────────────────────────────
CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT
);

-- ── Products ────────────────────────────────────────────────────
CREATE TABLE products (
    id TEXT PRIMARY KEY,  -- UUID stored as text
    name TEXT NOT NULL,
    sku TEXT NOT NULL UNIQUE,
    price REAL NOT NULL DEFAULT 0,
    category_id INTEGER REFERENCES categories(id),
    tags TEXT DEFAULT '[]',  -- JSON array as text
    metadata TEXT DEFAULT '{}',  -- JSON object as text
    created_at TEXT DEFAULT (datetime('now'))
);

-- ── Users (unicode identifiers) ─────────────────────────────────
CREATE TABLE "Ünïcödé Üsers" (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    "émâil" TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_active INTEGER DEFAULT 1,  -- SQLite boolean as integer
    created_at TEXT DEFAULT (datetime('now'))
);

-- ── Orders ──────────────────────────────────────────────────────
CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    total REAL NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- ── Order items (composite PK) ──────────────────────────────────
CREATE TABLE order_items (
    order_id INTEGER NOT NULL REFERENCES orders(id),
    product_id TEXT NOT NULL REFERENCES products(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price REAL NOT NULL,
    PRIMARY KEY (order_id, product_id)
);

-- ── Table without PK ────────────────────────────────────────────
CREATE TABLE audit_logs (
    event_type TEXT NOT NULL,
    payload TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- ── Empty table ─────────────────────────────────────────────────
CREATE TABLE empty_table (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label TEXT
);

-- ── Weird identifiers ───────────────────────────────────────────
CREATE TABLE "weird""name" (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    "col with spaces" TEXT,
    "SELECT" TEXT
);

-- ── Large text / blob ───────────────────────────────────────────
CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT,
    body TEXT,
    binary_data BLOB
);

-- ── Employees (no generated columns in SQLite) ──────────────────
CREATE TABLE employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    hire_date TEXT NOT NULL DEFAULT (date('now'))
);

-- ── Views ───────────────────────────────────────────────────────
CREATE VIEW active_users AS
SELECT id, "émâil", name FROM "Ünïcödé Üsers" WHERE is_active = 1;

CREATE VIEW order_summary AS
SELECT
    o.id AS order_id,
    o.status,
    o.total,
    COUNT(oi.product_id) AS item_count
FROM orders o
LEFT JOIN order_items oi ON oi.order_id = o.id
GROUP BY o.id, o.status, o.total;

-- ── Indexes ─────────────────────────────────────────────────────
CREATE INDEX idx_products_category ON products(category_id);
CREATE INDEX idx_orders_user ON orders(user_id);
CREATE INDEX idx_orders_created ON orders(created_at DESC);

-- ── Trigger ─────────────────────────────────────────────────────
CREATE TRIGGER products_updated_at
    AFTER UPDATE ON products
    FOR EACH ROW
BEGIN
    UPDATE products SET created_at = datetime('now') WHERE id = NEW.id;
END;

-- ════════════════════════════════════════════════════════════════
-- SEED DATA
-- ════════════════════════════════════════════════════════════════

INSERT INTO categories (name, description) VALUES
    ('Electronics', 'Electronic devices and accessories'),
    ('Books', 'Physical and digital books'),
    ('Clothing', 'Apparel and fashion'),
    ('Food & Beverage', NULL);

INSERT INTO products (id, name, sku, price, category_id, tags, metadata) VALUES
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'Laptop Pro', 'ELEC-001', 1299.99, 1,
     '["laptop", "computer"]', '{"brand": "TechCo", "weight_kg": 1.8}'),
    ('b1ffcd00-ad1c-4f99-cc7e-7cc0ce491b22', 'SQL Mastery', 'BOOK-001', 49.99, 2,
     '["database", "programming"]', '{"pages": 512, "format": "hardcover"}'),
    ('c2aade11-be2d-4aa0-dd8f-8dd1df502c33', 'Cotton T-Shirt', 'CLTH-001', 19.99, 3,
     '["cotton", "casual"]', '{"sizes": ["S", "M", "L", "XL"]}'),
    ('d3bbef22-cf3e-4bb1-ee9a-9ee2ea613d44', 'Green Tea', 'FOOD-001', 12.50, 4,
     '["tea", "organic"]', '{"origin": "Japan", "caffeine_mg": 30}');

INSERT INTO "Ünïcödé Üsers" ("émâil", name, is_active) VALUES
    ('alice@example.com', 'Alice Ün', 1),
    ('bob@example.com', 'Bob Smith', 1),
    ('carol@example.com', 'Carol Žert', 0),
    ('dave@example.com', 'Dàve Müller', 1);

INSERT INTO orders (user_id, status, total, notes) VALUES
    (1, 'delivered', 1349.98, 'First order'),
    (2, 'processing', 49.99, NULL),
    (1, 'pending', 32.49, 'Gift wrap please'),
    (4, 'shipped', 12.50, NULL);

INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES
    (1, 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 1, 1299.99),
    (1, 'd3bbef22-cf3e-4bb1-ee9a-9ee2ea613d44', 1, 12.50),
    (2, 'b1ffcd00-ad1c-4f99-cc7e-7cc0ce491b22', 1, 49.99),
    (3, 'c2aade11-be2d-4aa0-dd8f-8dd1df502c33', 1, 19.99),
    (3, 'd3bbef22-cf3e-4bb1-ee9a-9ee2ea613d44', 1, 12.50),
    (4, 'd3bbef22-cf3e-4bb1-ee9a-9ee2ea613d44', 1, 12.50);

INSERT INTO audit_logs (event_type, payload) VALUES
    ('user.created', '{"user_id": 1}'),
    ('order.created', '{"order_id": 1}'),
    ('order.shipped', '{"order_id": 4}');

INSERT INTO documents (title, body, binary_data) VALUES
    ('README', 'This is a long document. ', X'48656C6C6F'),
    ('Empty Doc', '', NULL),
    ('Binary Only', NULL, X'DEADBEEF');

INSERT INTO employees (first_name, last_name, hire_date) VALUES
    ('John', 'Doe', '2024-01-15'),
    ('Jane', 'Smith', '2024-03-20'),
    ('Bob', 'Johnson', '2023-11-01');

INSERT INTO "weird""name" ("col with spaces", "SELECT") VALUES
    ('hello', 'world'),
    (NULL, 'not a query');
