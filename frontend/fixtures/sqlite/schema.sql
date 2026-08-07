-- DB Pro E2E Fixture Schema — SQLite
-- Mirrors the PostgreSQL fixture for cross-dialect testing.

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    email       TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    bio         TEXT,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Categories (self-referencing)
CREATE TABLE IF NOT EXISTS categories (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL UNIQUE,
    parent_id   INTEGER REFERENCES categories(id)
);

-- Products
CREATE TABLE IF NOT EXISTS products (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    description   TEXT,
    price         REAL NOT NULL DEFAULT 0.00,
    category_id   INTEGER REFERENCES categories(id),
    in_stock      INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Orders
CREATE TABLE IF NOT EXISTS orders (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    status      TEXT NOT NULL DEFAULT 'pending',
    total       REAL NOT NULL DEFAULT 0.00,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    shipped_at  TEXT
);

-- Order items
CREATE TABLE IF NOT EXISTS order_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id    INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id  INTEGER NOT NULL REFERENCES products(id),
    quantity    INTEGER NOT NULL DEFAULT 1,
    unit_price  REAL NOT NULL,
    UNIQUE (order_id, product_id)
);

-- Tags
CREATE TABLE IF NOT EXISTS tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT NOT NULL UNIQUE
);

-- Product-Tags junction
CREATE TABLE IF NOT EXISTS product_tags (
    product_id  INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, tag_id)
);

-- Audit log
CREATE TABLE IF NOT EXISTS audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id   INTEGER NOT NULL,
    action      TEXT NOT NULL,
    old_value   TEXT,
    new_value   TEXT,
    changed_by  INTEGER REFERENCES users(id),
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes
CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_is_active ON users (is_active);
CREATE INDEX idx_products_category ON products (category_id);
CREATE INDEX idx_products_price ON products (price);
CREATE INDEX idx_orders_user ON orders (user_id);
CREATE INDEX idx_orders_status ON orders (status);
CREATE INDEX idx_order_items_order ON order_items (order_id);
CREATE INDEX idx_audit_log_entity ON audit_log (entity_type, entity_id);
CREATE INDEX idx_audit_log_created ON audit_log (created_at);

-- Views
CREATE VIEW IF NOT EXISTS active_users_summary AS
SELECT
    u.id,
    u.email,
    u.name,
    COUNT(o.id) AS order_count,
    COALESCE(SUM(o.total), 0) AS total_spent
FROM users u
LEFT JOIN orders o ON o.user_id = u.id
WHERE u.is_active = 1
GROUP BY u.id, u.email, u.name;

CREATE VIEW IF NOT EXISTS product_catalog AS
SELECT
    p.id,
    p.name AS product_name,
    p.price,
    p.in_stock,
    c.name AS category_name
FROM products p
LEFT JOIN categories c ON c.id = p.category_id;
