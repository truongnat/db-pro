-- PostgreSQL fixture seed data.

-- Categories
INSERT INTO categories (name, description) VALUES
    ('Electronics', 'Electronic devices and accessories'),
    ('Books', 'Physical and digital books'),
    ('Clothing', 'Apparel and fashion'),
    ('Food & Beverage', NULL);

-- Products
INSERT INTO products (id, name, sku, price, category_id, tags, metadata) VALUES
    ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'Laptop Pro', 'ELEC-001', 1299.99, 1,
     ARRAY['laptop', 'computer'], '{"brand": "TechCo", "weight_kg": 1.8}'),
    ('b1ffcd00-ad1c-4f99-cc7e-7cc0ce491b22', 'SQL Mastery', 'BOOK-001', 49.99, 2,
     ARRAY['database', 'programming'], '{"pages": 512, "format": "hardcover"}'),
    ('c2aade11-be2d-6ab0-dd8f-8dd1df502c33', 'Cotton T-Shirt', 'CLTH-001', 19.99, 3,
     ARRAY['cotton', 'casual'], '{"sizes": ["S", "M", "L", "XL"]}'),
    ('d3aaef22-cf3e-7a11-ee9a-9ee2ea613d44', 'Green Tea', 'FOOD-001', 12.50, 4,
     ARRAY['tea', 'organic'], '{"origin": "Japan", "caffeine_mg": 30}');

-- Unicode users
INSERT INTO "Ünïcödé Üsers" ("émâil", name, is_active) VALUES
    ('alice@example.com', 'Alice Ün', true),
    ('bob@example.com', 'Bob Smith', true),
    ('carol@example.com', 'Carol Žert', false),
    ('dave@example.com', 'Dàve Müller', true);

-- Orders
INSERT INTO orders (user_id, status, total, notes) VALUES
    (1, 'delivered', 1349.98, 'First order'),
    (2, 'processing', 49.99, NULL),
    (1, 'pending', 32.49, 'Gift wrap please'),
    (4, 'shipped', 12.50, NULL);

-- Order items
INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES
    (1, 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 1, 1299.99),
    (1, 'd3aaef22-cf3e-7a11-ee9a-9ee2ea613d44', 1, 12.50),
    (2, 'b1ffcd00-ad1c-4f99-cc7e-7cc0ce491b22', 1, 49.99),
    (3, 'c2aade11-be2d-6ab0-dd8f-8dd1df502c33', 1, 19.99),
    (3, 'd3aaef22-cf3e-7a11-ee9a-9ee2ea613d44', 1, 12.50),
    (4, 'd3aaef22-cf3e-7a11-ee9a-9ee2ea613d44', 1, 12.50);

-- Audit logs
INSERT INTO audit_logs (event_type, payload) VALUES
    ('user.created', '{"user_id": 1}'),
    ('order.created', '{"order_id": 1}'),
    ('order.shipped', '{"order_id": 4}');

-- Documents with large text and binary
INSERT INTO documents (title, body, binary_data) VALUES
    ('README', repeat('This is a long document. ', 1000), E'\\x48656c6c6f'),
    ('Empty Doc', '', NULL),
    ('Binary Only', NULL, E'\\xDEADBEEF');

-- Employees
INSERT INTO employees (first_name, last_name, hire_date) VALUES
    ('John', 'Doe', '2024-01-15'),
    ('Jane', 'Smith', '2024-03-20'),
    ('Bob', 'Johnson', '2023-11-01');

-- Weird name table
INSERT INTO "weird""name" ("col with spaces", "SELECT") VALUES
    ('hello', 'world'),
    (NULL, 'not a query');

-- Decoder matrix rows: one fully populated row per class, and one row that is NULL
-- in every nullable column (id and flag are NOT NULL by DDL).
INSERT INTO decoder_matrix (
    id, flag, count, big_count, ratio, precise_ratio, amount,
    calendar_date, wall_time, zoned_time, local_stamp, instant, span,
    token, doc, payload, blob, address, network, status, quantity, postal,
    labels, slot, pair, missing
) VALUES (
    1, TRUE, 2147483647, 9223372036854775807, 1.5, 0.1, 12345678901234567890.1234,
    '2024-03-15', '10:20:30.123456', '10:20:30.123456+07:00',
    '2024-03-15 10:20:30.123456', '2024-03-15 10:20:30.123456+00:00',
    '1 mons 2 days 03:04:05.000006',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    '{"a": 1, "b": [true, null]}', '{"brand": "TechCo", "weight_kg": 1.8}',
    E'\\xDEADBEEF', '192.0.2.1', '192.0.2.0/24',
    'shipped', 7, 'SW1A 1AA',
    ARRAY['laptop', 'computer'], '[1,10)', ROW(1, 'x'), NULL
), (
    2, FALSE, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL,
    NULL, NULL, NULL, NULL
);
