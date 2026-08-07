-- DB Pro E2E Fixture Seed Data — PostgreSQL
-- Realistic sample data for end-to-end testing.

-- Users
INSERT INTO public.users (email, name, bio, is_active) VALUES
    ('alice@example.com', 'Alice Johnson', 'Software engineer', true),
    ('bob@example.com', 'Bob Smith', 'Product manager', true),
    ('carol@example.com', 'Carol Williams', 'Data analyst', true),
    ('dave@example.com', 'Dave Brown', 'Designer', false),
    ('eve@example.com', 'Eve Davis', 'DevOps engineer', true);

-- Categories
INSERT INTO public.categories (name, slug, parent_id) VALUES
    ('Electronics', 'electronics', NULL),
    ('Books', 'books', NULL),
    ('Clothing', 'clothing', NULL),
    ('Laptops', 'laptops', 1),
    ('Phones', 'phones', 1),
    ('Fiction', 'fiction', 2),
    ('Non-Fiction', 'non-fiction', 2);

-- Products
INSERT INTO public.products (name, description, price, category_id, in_stock) VALUES
    ('Laptop Pro 15', 'High-performance laptop', 1299.99, 4, true),
    ('Smartphone X', 'Latest smartphone', 899.00, 5, true),
    ('SQL Mastery', 'Complete guide to SQL', 49.99, 7, true),
    ('Rust Programming', 'Learn Rust from scratch', 39.99, 7, true),
    ('Winter Jacket', 'Warm winter jacket', 129.99, 3, true),
    ('USB-C Cable', 'Fast charging cable', 12.99, 1, false),
    ('Running Shoes', 'Lightweight running shoes', 89.99, 3, true),
    ('Tablet Air', 'Lightweight tablet', 599.00, 4, true);

-- Tags
INSERT INTO public.tags (name) VALUES
    ('bestseller'), ('new'), ('sale'), ('premium'), ('eco-friendly');

-- Product tags
INSERT INTO public.product_tags (product_id, tag_id) VALUES
    (1, 1), (1, 4),  -- Laptop Pro: bestseller, premium
    (2, 2), (2, 4),  -- Smartphone X: new, premium
    (3, 1),           -- SQL Mastery: bestseller
    (4, 2),           -- Rust Programming: new
    (5, 3),           -- Winter Jacket: sale
    (7, 5),           -- Running Shoes: eco-friendly
    (8, 2), (8, 4);  -- Tablet Air: new, premium

-- Orders
INSERT INTO public.orders (user_id, status, total, created_at, shipped_at) VALUES
    (1, 'completed', 1349.98, '2025-01-15 10:30:00', '2025-01-16 09:00:00'),
    (2, 'completed', 89.99, '2025-02-01 14:20:00', '2025-02-02 11:00:00'),
    (1, 'shipped', 599.00, '2025-03-10 08:15:00', '2025-03-12 16:30:00'),
    (3, 'pending', 179.98, '2025-04-01 16:45:00', NULL),
    (5, 'completed', 49.99, '2025-04-15 12:00:00', '2025-04-16 10:00:00'),
    (2, 'cancelled', 1299.99, '2025-05-01 09:30:00', NULL);

-- Order items
INSERT INTO public.order_items (order_id, product_id, quantity, unit_price) VALUES
    (1, 1, 1, 1299.99),   -- Alice buys Laptop
    (1, 4, 1, 49.99),     -- Alice buys Rust book (but total = 1349.98)
    (2, 7, 1, 89.99),     -- Bob buys Running Shoes
    (3, 8, 1, 599.00),    -- Alice buys Tablet
    (4, 5, 1, 129.99),    -- Carol buys Jacket
    (4, 3, 1, 49.99),     -- Carol buys SQL Mastery
    (5, 3, 1, 49.99),     -- Eve buys SQL Mastery
    (6, 1, 1, 1299.99);   -- Bob buys Laptop (cancelled)

-- Audit log entries
INSERT INTO public.audit_log (entity_type, entity_id, action, old_value, new_value, changed_by) VALUES
    ('user', 4, 'update',
     '{"is_active": true}', '{"is_active": false}', 1),
    ('product', 6, 'update',
     '{"in_stock": true}', '{"in_stock": false}', 1),
    ('order', 6, 'update',
     '{"status": "pending"}', '{"status": "cancelled"}', 2);
