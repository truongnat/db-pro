-- DB Pro E2E Fixture Seed Data — SQLite
-- Same data as PostgreSQL fixture for cross-dialect comparison.

-- Users
INSERT INTO users (email, name, bio, is_active) VALUES
    ('alice@example.com', 'Alice Johnson', 'Software engineer', 1),
    ('bob@example.com', 'Bob Smith', 'Product manager', 1),
    ('carol@example.com', 'Carol Williams', 'Data analyst', 1),
    ('dave@example.com', 'Dave Brown', 'Designer', 0),
    ('eve@example.com', 'Eve Davis', 'DevOps engineer', 1);

-- Categories
INSERT INTO categories (name, slug, parent_id) VALUES
    ('Electronics', 'electronics', NULL),
    ('Books', 'books', NULL),
    ('Clothing', 'clothing', NULL),
    ('Laptops', 'laptops', 1),
    ('Phones', 'phones', 1),
    ('Fiction', 'fiction', 2),
    ('Non-Fiction', 'non-fiction', 2);

-- Products
INSERT INTO products (name, description, price, category_id, in_stock) VALUES
    ('Laptop Pro 15', 'High-performance laptop', 1299.99, 4, 1),
    ('Smartphone X', 'Latest smartphone', 899.00, 5, 1),
    ('SQL Mastery', 'Complete guide to SQL', 49.99, 7, 1),
    ('Rust Programming', 'Learn Rust from scratch', 39.99, 7, 1),
    ('Winter Jacket', 'Warm winter jacket', 129.99, 3, 1),
    ('USB-C Cable', 'Fast charging cable', 12.99, 1, 0),
    ('Running Shoes', 'Lightweight running shoes', 89.99, 3, 1),
    ('Tablet Air', 'Lightweight tablet', 599.00, 4, 1);

-- Tags
INSERT INTO tags (name) VALUES
    ('bestseller'), ('new'), ('sale'), ('premium'), ('eco-friendly');

-- Product tags
INSERT INTO product_tags (product_id, tag_id) VALUES
    (1, 1), (1, 4),
    (2, 2), (2, 4),
    (3, 1),
    (4, 2),
    (5, 3),
    (7, 5),
    (8, 2), (8, 4);

-- Orders
INSERT INTO orders (user_id, status, total, created_at, shipped_at) VALUES
    (1, 'completed', 1349.98, '2025-01-15 10:30:00', '2025-01-16 09:00:00'),
    (2, 'completed', 89.99, '2025-02-01 14:20:00', '2025-02-02 11:00:00'),
    (1, 'shipped', 599.00, '2025-03-10 08:15:00', '2025-03-12 16:30:00'),
    (3, 'pending', 179.98, '2025-04-01 16:45:00', NULL),
    (5, 'completed', 49.99, '2025-04-15 12:00:00', '2025-04-16 10:00:00'),
    (2, 'cancelled', 1299.99, '2025-05-01 09:30:00', NULL);

-- Order items
INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES
    (1, 1, 1, 1299.99),
    (1, 4, 1, 49.99),
    (2, 7, 1, 89.99),
    (3, 8, 1, 599.00),
    (4, 5, 1, 129.99),
    (4, 3, 1, 49.99),
    (5, 3, 1, 49.99),
    (6, 1, 1, 1299.99);

-- Audit log
INSERT INTO audit_log (entity_type, entity_id, action, old_value, new_value, changed_by) VALUES
    ('user', 4, 'update',
     '{"is_active": true}', '{"is_active": false}', 1),
    ('product', 6, 'update',
     '{"in_stock": true}', '{"in_stock": false}', 1),
    ('order', 6, 'update',
     '{"status": "pending"}', '{"status": "cancelled"}', 2);
