-- P2 Hardening — PostgreSQL fixture seed data
-- Generates 500+ users, 500+ products, 1000+ orders, 10k+ sessions/audit_logs.
-- Deterministic: uses generate_series with seeded patterns.

-- ══════════════════════════════════════════════════════════════
-- ORGANIZATIONS (20)
-- ══════════════════════════════════════════════════════════════

INSERT INTO organizations (name, slug, plan, is_active, metadata) VALUES
    ('Acme Corp', 'acme-corp', 'enterprise', true, '{"employees": 500, "country": "US"}'),
    ('Globex Inc', 'globex-inc', 'business', true, '{"employees": 200, "country": "UK"}'),
    ('Initech', 'initech', 'business', true, '{"employees": 150, "country": "US"}'),
    ('Umbrella Corp', 'umbrella-corp', 'enterprise', true, '{"employees": 1000, "country": "JP"}'),
    ('Stark Industries', 'stark-industries', 'enterprise', true, '{"employees": 800, "country": "US"}'),
    ('Wayne Enterprises', 'wayne-enterprises', 'enterprise', true, '{"employees": 600, "country": "US"}'),
    ('Cyberdyne Systems', 'cyberdyne-systems', 'business', true, '{"employees": 300, "country": "US"}'),
    ('Soylent Corp', 'soylent-corp', 'starter', true, '{"employees": 50, "country": "US"}'),
    ('Weyland-Yutani', 'weyland-yutani', 'enterprise', true, '{"employees": 2000, "country": "UK"}'),
    ('Tyrell Corp', 'tyrell-corp', 'business', true, '{"employees": 400, "country": "US"}'),
    ('Oscorp', 'oscorp', 'business', false, '{"employees": 250, "country": "US"}'),
    ('LexCorp', 'lexcorp', 'enterprise', true, '{"employees": 700, "country": "US"}'),
    ('Aperture Science', 'aperture-science', 'business', true, '{"employees": 180, "country": "US"}'),
    ('Massive Dynamic', 'massive-dynamic', 'enterprise', true, '{"employees": 900, "country": "US"}'),
    ('Wonka Industries', 'wonka-industries', 'starter', true, '{"employees": 30, "country": "UK"}'),
    ('Dunder Mifflin', 'dunder-mifflin', 'starter', true, '{"employees": 20, "country": "US"}'),
    ('Pied Piper', 'pied-piper', 'starter', false, '{"employees": 10, "country": "US"}'),
    ('Hooli', 'hooli', 'enterprise', true, '{"employees": 5000, "country": "US"}'),
    ('Bluth Company', 'bluth-company', 'free', true, '{"employees": 5, "country": "US"}'),
    ('Sterling Cooper', 'sterling-cooper', 'business', true, '{"employees": 100, "country": "US"}');

-- ══════════════════════════════════════════════════════════════
-- USERS (600 — UUID PK, various types)
-- ══════════════════════════════════════════════════════════════

INSERT INTO users (email, username, display_name, password_hash, is_active, last_login_at, preferences)
SELECT
    format('user%s@example.com', gs) AS email,
    format('user_%s', gs) AS username,
    format('User Number %s', gs) AS display_name,
    md5(format('password-%s-%s', gs, 'salt')) AS password_hash,
    (gs % 10 != 0) AS is_active,  -- 90% active
    CASE WHEN gs % 3 = 0 THEN now() - (gs || ' hours')::interval ELSE NULL END AS last_login_at,
    jsonb_build_object(
        'theme', CASE WHEN gs % 2 = 0 THEN 'dark' ELSE 'light' END,
        'language', CASE (gs % 5) WHEN 0 THEN 'en' WHEN 1 THEN 'vi' WHEN 2 THEN 'ja' WHEN 3 THEN 'fr' ELSE 'de' END,
        'notifications', gs % 4 != 0
    ) AS preferences
FROM generate_series(1, 600) AS gs;

-- ══════════════════════════════════════════════════════════════
-- PROFILES (500 — 1:1 with users, some missing)
-- ══════════════════════════════════════════════════════════════

INSERT INTO profiles (user_id, bio, avatar_url, phone, location, date_of_birth, website, social_links)
SELECT
    sub.user_id,
    CASE WHEN sub.rn % 3 = 0 THEN format('Bio for user %s. Loves coding and databases.', sub.rn) ELSE NULL END,
    CASE WHEN sub.rn % 2 = 0 THEN format('https://avatars.example.com/user%s.png', sub.rn) ELSE NULL END,
    CASE WHEN sub.rn % 4 = 0 THEN format('+1-555-%s', lpad(sub.rn::text, 7, '0')) ELSE NULL END,
    CASE (sub.rn % 6) WHEN 0 THEN 'New York, US' WHEN 1 THEN 'London, UK' WHEN 2 THEN 'Tokyo, JP'
        WHEN 3 THEN 'Berlin, DE' WHEN 4 THEN 'Hanoi, VN' ELSE NULL END,
    DATE '1970-01-01' + ((sub.rn * 7) % 12000 || ' days')::interval,
    CASE WHEN sub.rn % 5 = 0 THEN format('https://user%s.example.com', sub.rn) ELSE NULL END,
    CASE WHEN sub.rn % 3 = 0 THEN jsonb_build_object('twitter', format('@user%s', sub.rn), 'github', format('gh-user%s', sub.rn)) ELSE '{}' END
FROM (
    SELECT u.id AS user_id, row_number() OVER (ORDER BY u.id) AS rn
    FROM users u
    ORDER BY u.id
    LIMIT 500
) sub;

-- ══════════════════════════════════════════════════════════════
-- ORGANIZATION MEMBERS (~800)
-- ══════════════════════════════════════════════════════════════

INSERT INTO organization_members (organization_id, user_id, role)
SELECT
    ((row_number - 1) % 20) + 1 AS organization_id,
    user_id,
    CASE (row_number % 4)
        WHEN 0 THEN 'owner'::membership_role
        WHEN 1 THEN 'admin'::membership_role
        WHEN 2 THEN 'member'::membership_role
        ELSE 'viewer'::membership_role
    END AS role
FROM (
    SELECT u.id AS user_id, row_number() OVER (ORDER BY u.id) AS row_number
    FROM users u
    LIMIT 800
) sub;

-- ══════════════════════════════════════════════════════════════
-- CATEGORIES (50, with hierarchy)
-- ══════════════════════════════════════════════════════════════

INSERT INTO categories (name, slug, parent_id, description, sort_order, is_active) VALUES
    ('Electronics', 'electronics', NULL, 'Electronic devices and accessories', 1, true),
    ('Computers', 'computers', 1, 'Desktop and laptop computers', 1, true),
    ('Laptops', 'laptops', 2, 'Portable computers', 1, true),
    ('Desktops', 'desktops', 2, 'Desktop computers', 2, true),
    ('Phones', 'phones', 1, 'Mobile phones and accessories', 2, true),
    ('Audio', 'audio', 1, 'Headphones, speakers, audio equipment', 3, true),
    ('Books', 'books', NULL, 'Physical and digital books', 2, true),
    ('Programming', 'programming', 7, 'Programming and software development', 1, true),
    ('Database', 'database', 7, 'Database management and design', 2, true),
    ('Clothing', 'clothing', NULL, 'Apparel and fashion', 3, true),
    ('Food & Beverage', 'food-beverage', NULL, 'Food and drink items', 4, true),
    ('Home & Garden', 'home-garden', NULL, 'Home improvement and garden', 5, true),
    ('Sports', 'sports', NULL, 'Sports and outdoor equipment', 6, true),
    ('Toys', 'toys', NULL, 'Toys and games', 7, true),
    ('Automotive', 'automotive', NULL, 'Car parts and accessories', 8, true),
    ('Health', 'health', NULL, 'Health and wellness', 9, true),
    ('Office', 'office', NULL, 'Office supplies', 10, true),
    ('Software', 'software', NULL, 'Software licenses and subscriptions', 11, true),
    ('Services', 'services', NULL, 'Professional services', 12, true),
    ('Other', 'other', NULL, 'Miscellaneous items', 99, true);

-- Generate more subcategories to reach 50
INSERT INTO categories (name, slug, parent_id, description, sort_order, is_active)
SELECT
    format('%s Sub-%s', parent.name, gs) AS name,
    format('%s-sub-%s', parent.slug, gs) AS slug,
    parent.id AS parent_id,
    format('Subcategory %s of %s', gs, parent.name) AS description,
    gs AS sort_order,
    (gs % 10 != 0) AS is_active  -- 90% active
FROM generate_series(1, 30) AS gs
CROSS JOIN LATERAL (
    SELECT c.id, c.name, c.slug FROM categories c WHERE c.parent_id IS NULL ORDER BY c.id LIMIT 1 OFFSET (gs % 12)
) parent;

-- ══════════════════════════════════════════════════════════════
-- PRODUCTS (600 — UUID PK, various types)
-- ══════════════════════════════════════════════════════════════

INSERT INTO products (name, sku, slug, description, price, cost, category_id, tags, attributes, is_active, stock_quantity)
SELECT
    format('Product %s — %s', gs, (ARRAY['Premium', 'Basic', 'Pro', 'Lite', 'Ultra'])[1 + (gs % 5)]) AS name,
    format('SKU-%s', lpad(gs::text, 6, '0')) AS sku,
    format('product-%s', lpad(gs::text, 6, '0')) AS slug,
    CASE WHEN gs % 7 = 0 THEN NULL ELSE format('Description for product %s with great features.', gs) END,
    (random() * 999 + 1)::numeric(12, 2) AS price,
    (random() * 500 + 0.5)::numeric(12, 2) AS cost,
    ((gs % 20) + 1) AS category_id,
    ARRAY[
        (ARRAY['electronics', 'books', 'clothing', 'food', 'sports'])[1 + (gs % 5)],
        CASE WHEN gs % 2 = 0 THEN 'sale' ELSE 'new' END,
        CASE WHEN gs % 3 = 0 THEN 'featured' ELSE 'regular' END
    ] AS tags,
    jsonb_build_object(
        'weight_kg', (random() * 10)::numeric(6, 2),
        'color', (ARRAY['red', 'blue', 'green', 'black', 'white'])[1 + (gs % 5)],
        'rating', (random() * 4 + 1)::numeric(3, 1)
    ) AS attributes,
    (gs % 15 != 0) AS is_active,  -- ~93% active
    (gs * 7 % 200) AS stock_quantity
FROM generate_series(1, 600) AS gs;

-- ══════════════════════════════════════════════════════════════
-- PRODUCT CATEGORIES (many-to-many, ~800)
-- ══════════════════════════════════════════════════════════════

INSERT INTO product_categories (product_id, category_id, is_primary)
SELECT
    p.id,
    ((gs % 20) + 1) AS category_id,
    (gs % 3 = 0) AS is_primary
FROM (SELECT id, row_number() OVER (ORDER BY id) AS rn FROM products) p
CROSS JOIN generate_series(1, 2) AS extra(gs)
WHERE p.rn % 3 = 0  -- 1/3 of products have multiple categories
LIMIT 400;

-- ══════════════════════════════════════════════════════════════
-- ORDERS (1200 — bigserial PK, FK to users)
-- ══════════════════════════════════════════════════════════════

INSERT INTO orders (user_id, organization_id, status, subtotal, tax, discount, total, shipping_address, notes)
SELECT
    (SELECT id FROM users ORDER BY id OFFSET (gs % 600) LIMIT 1) AS user_id,
    CASE WHEN gs % 3 = 0 THEN ((gs % 20) + 1) ELSE NULL END AS organization_id,
    (ARRAY['pending', 'processing', 'shipped', 'delivered', 'cancelled'])[1 + (gs % 5)]::order_status AS status,
    (random() * 500 + 10)::numeric(12, 2) AS subtotal,
    (random() * 50)::numeric(12, 2) AS tax,
    CASE WHEN gs % 4 = 0 THEN (random() * 30)::numeric(12, 2) ELSE 0 END AS discount,
    (random() * 600 + 10)::numeric(12, 2) AS total,
    CASE WHEN gs % 2 = 0 THEN jsonb_build_object(
        'street', format('%s Main St', gs),
        'city', (ARRAY['New York', 'London', 'Tokyo', 'Berlin', 'Hanoi'])[1 + (gs % 5)],
        'zip', format('%05d', gs % 100000),
        'country', (ARRAY['US', 'UK', 'JP', 'DE', 'VN'])[1 + (gs % 5)]
    ) ELSE NULL END AS shipping_address,
    CASE WHEN gs % 5 = 0 THEN format('Order note #%s', gs) ELSE NULL END AS notes
FROM generate_series(1, 1200) AS gs;

-- Fix: set created_at with spread over 6 months
UPDATE orders SET created_at = now() - ((id % 180) || ' days')::interval - ((id % 24) || ' hours')::interval;
UPDATE orders SET updated_at = created_at + ((id % 10) || ' days')::interval;

-- ══════════════════════════════════════════════════════════════
-- ORDER ITEMS (~2400 — composite PK)
-- ══════════════════════════════════════════════════════════════

INSERT INTO order_items (order_id, product_id, quantity, unit_price, discount)
SELECT
    o.order_id,
    p.id AS product_id,
    (gs % 5 + 1) AS quantity,
    p.price AS unit_price,
    CASE WHEN gs % 6 = 0 THEN (p.price * 0.1)::numeric(12, 2) ELSE 0 END AS discount
FROM (
    SELECT id AS order_id, row_number() OVER (ORDER BY id) AS rn FROM orders
) o
CROSS JOIN LATERAL (
    SELECT id, price FROM products ORDER BY id OFFSET (o.rn % 600) LIMIT 1
) p
CROSS JOIN generate_series(1, 2) AS gs
WHERE o.rn % 3 != 0;  -- 2/3 of orders have items via this generator

-- ══════════════════════════════════════════════════════════════
-- SETTINGS (30 key-value pairs)
-- ══════════════════════════════════════════════════════════════

INSERT INTO settings (key, value, description) VALUES
    ('app.name', '"DB Pro"', 'Application name'),
    ('app.version', '"0.1.0"', 'Application version'),
    ('app.max_connections', '50', 'Maximum concurrent connections'),
    ('app.query_timeout', '30000', 'Query timeout in milliseconds'),
    ('app.theme.default', '"dark"', 'Default theme'),
    ('app.language.default', '"en"', 'Default language'),
    ('app.export.max_rows', '100000', 'Maximum export rows'),
    ('app.export.formats', '["csv", "json", "xlsx"]', 'Supported export formats'),
    ('auth.session_timeout', '3600', 'Session timeout in seconds'),
    ('auth.max_attempts', '5', 'Max login attempts before lockout'),
    ('auth.lockout_duration', '900', 'Lockout duration in seconds'),
    ('backup.enabled', 'true', 'Whether backups are enabled'),
    ('backup.schedule', '"0 2 * * *"', 'Backup cron schedule'),
    ('backup.retention_days', '30', 'Backup retention in days'),
    ('logging.level', '"info"', 'Log level'),
    ('logging.retention_days', '90', 'Log retention in days'),
    ('feature.agent_enabled', 'false', 'Whether agent feature is enabled'),
    ('feature.mcp_enabled', 'false', 'Whether MCP is enabled'),
    ('feature.er_diagram', 'true', 'Whether ER diagram is available'),
    ('feature.export', 'true', 'Whether export is available');

-- ══════════════════════════════════════════════════════════════
-- AUDIT LOGS (10,000 — no PK, for virtualization testing)
-- ══════════════════════════════════════════════════════════════

INSERT INTO audit_logs (action, entity_type, entity_id, actor_user_id, details, ip_address, created_at)
SELECT
    (ARRAY['create', 'update', 'delete', 'login', 'logout', 'export'])[1 + (gs % 6)]::audit_action AS action,
    (ARRAY['user', 'order', 'product', 'organization', 'session', 'setting'])[1 + (gs % 6)] AS entity_type,
    CASE WHEN gs % 3 = 0 THEN gs::text ELSE NULL END AS entity_id,
    CASE WHEN gs % 2 = 0 THEN (SELECT id FROM users ORDER BY id OFFSET (gs % 600) LIMIT 1) ELSE NULL END AS actor_user_id,
    jsonb_build_object(
        'detail_id', gs,
        'action', (ARRAY['create', 'update', 'delete', 'login', 'logout', 'export'])[1 + (gs % 6)],
        'metadata', format('batch-%s', gs / 100)
    ) AS details,
    format('192.168.%s.%s', (gs / 256) % 256, gs % 256) AS ip_address,
    now() - ((gs * 3) % 4320 || ' hours')::interval AS created_at  -- spread over 6 months
FROM generate_series(1, 10000) AS gs;

-- ══════════════════════════════════════════════════════════════
-- SESSIONS (5,000 — UUID PK, for virtualization testing)
-- ══════════════════════════════════════════════════════════════

INSERT INTO sessions (user_id, token_hash, ip_address, user_agent, expires_at, created_at)
SELECT
    (SELECT id FROM users ORDER BY id OFFSET (gs % 600) LIMIT 1) AS user_id,
    md5(format('session-token-%s-%s', gs, 'secret')) AS token_hash,
    format('10.%s.%s.%s', (gs / 65536) % 256, (gs / 256) % 256, gs % 256) AS ip_address,
    (ARRAY[
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)',
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
        'Mozilla/5.0 (X11; Linux x86_64)',
        'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)',
        'Mozilla/5.0 (iPad; CPU OS 17_0)'
    ])[1 + (gs % 5)] AS user_agent,
    now() + ((gs % 168) || ' hours')::interval AS expires_at,  -- expires within 7 days
    now() - ((gs % 72) || ' hours')::interval AS created_at  -- created within 3 days
FROM generate_series(1, 5000) AS gs;

-- ══════════════════════════════════════════════════════════════
-- DOCUMENTS (100)
-- ══════════════════════════════════════════════════════════════

INSERT INTO documents (title, body, binary_data)
SELECT
    format('Document %s', gs) AS title,
    CASE
        WHEN gs % 4 = 0 THEN NULL
        WHEN gs % 3 = 0 THEN ''
        ELSE repeat(format('Paragraph %s: Lorem ipsum dolor sit amet. ', gs), 50)
    END AS body,
    CASE WHEN gs % 5 = 0 THEN decode(md5(gs::text), 'hex') ELSE NULL END AS binary_data
FROM generate_series(1, 100) AS gs;

-- ══════════════════════════════════════════════════════════════
-- EMPLOYEES (200)
-- ══════════════════════════════════════════════════════════════

INSERT INTO employees (first_name, last_name, email, hire_date, salary, department)
SELECT
    (ARRAY['Alice', 'Bob', 'Carol', 'Dave', 'Eve', 'Frank', 'Grace', 'Hank', 'Ivy', 'Jack',
           'Karen', 'Leo', 'Mona', 'Nick', 'Olivia', 'Paul', 'Quinn', 'Rita', 'Sam', 'Tina'])[1 + (gs % 20)] AS first_name,
    format('Employee-%s', gs) AS last_name,
    format('emp%s@company.com', gs) AS email,
    DATE '2020-01-01' + ((gs * 3) % 1800 || ' days')::interval AS hire_date,
    CASE WHEN gs % 10 = 0 THEN NULL ELSE (30000 + (gs * 137 % 120000))::numeric(10, 2) END AS salary,
    (ARRAY['Engineering', 'Marketing', 'Sales', 'HR', 'Finance', 'Operations', 'Legal', 'Support'])[1 + (gs % 8)] AS department
FROM generate_series(1, 200) AS gs;

-- ══════════════════════════════════════════════════════════════
-- WEIRD NAME TABLE
-- ══════════════════════════════════════════════════════════════

INSERT INTO "weird""name" ("col with spaces", "SELECT") VALUES
    ('hello', 'world'),
    (NULL, 'not a query'),
    ('test', NULL);
