-- DB Pro E2E Fixture Schema — PostgreSQL
-- This schema creates a realistic database structure for end-to-end testing.

-- Users table: core entity
CREATE TABLE IF NOT EXISTS public.users (
    id          BIGSERIAL PRIMARY KEY,
    email       TEXT NOT NULL UNIQUE,
    name        VARCHAR(255) NOT NULL,
    bio         TEXT,
    is_active   BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Categories for products
CREATE TABLE IF NOT EXISTS public.categories (
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(100) NOT NULL,
    slug        VARCHAR(100) NOT NULL UNIQUE,
    parent_id   INTEGER REFERENCES public.categories(id)
);

-- Products table
CREATE TABLE IF NOT EXISTS public.products (
    id            BIGSERIAL PRIMARY KEY,
    name          VARCHAR(255) NOT NULL,
    description   TEXT,
    price         NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    category_id   INTEGER REFERENCES public.categories(id),
    in_stock      BOOLEAN NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Orders table
CREATE TABLE IF NOT EXISTS public.orders (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES public.users(id),
    status      VARCHAR(50) NOT NULL DEFAULT 'pending',
    total       NUMERIC(12, 2) NOT NULL DEFAULT 0.00,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    shipped_at  TIMESTAMPTZ
);

-- Order items (junction table)
CREATE TABLE IF NOT EXISTS public.order_items (
    id          BIGSERIAL PRIMARY KEY,
    order_id    BIGINT NOT NULL REFERENCES public.orders(id) ON DELETE CASCADE,
    product_id  BIGINT NOT NULL REFERENCES public.products(id),
    quantity    INTEGER NOT NULL DEFAULT 1,
    unit_price  NUMERIC(10, 2) NOT NULL,
    UNIQUE (order_id, product_id)
);

-- Tags for flexible labeling
CREATE TABLE IF NOT EXISTS public.tags (
    id    SERIAL PRIMARY KEY,
    name  VARCHAR(50) NOT NULL UNIQUE
);

-- Many-to-many: products <-> tags
CREATE TABLE IF NOT EXISTS public.product_tags (
    product_id  BIGINT NOT NULL REFERENCES public.products(id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES public.tags(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, tag_id)
);

-- Audit log
CREATE TABLE IF NOT EXISTS public.audit_log (
    id          BIGSERIAL PRIMARY KEY,
    entity_type VARCHAR(50) NOT NULL,
    entity_id   BIGINT NOT NULL,
    action      VARCHAR(20) NOT NULL,
    old_value   JSONB,
    new_value   JSONB,
    changed_by  BIGINT REFERENCES public.users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX idx_users_email ON public.users (email);
CREATE INDEX idx_users_is_active ON public.users (is_active);
CREATE INDEX idx_products_category ON public.products (category_id);
CREATE INDEX idx_products_price ON public.products (price);
CREATE INDEX idx_orders_user ON public.orders (user_id);
CREATE INDEX idx_orders_status ON public.orders (status);
CREATE INDEX idx_order_items_order ON public.order_items (order_id);
CREATE INDEX idx_audit_log_entity ON public.audit_log (entity_type, entity_id);
CREATE INDEX idx_audit_log_created ON public.audit_log (created_at);

-- View: active users with order count
CREATE OR REPLACE VIEW public.active_users_summary AS
SELECT
    u.id,
    u.email,
    u.name,
    COUNT(o.id) AS order_count,
    COALESCE(SUM(o.total), 0) AS total_spent
FROM public.users u
LEFT JOIN public.orders o ON o.user_id = u.id
WHERE u.is_active = true
GROUP BY u.id, u.email, u.name;

-- View: product catalog with category name
CREATE OR REPLACE VIEW public.product_catalog AS
SELECT
    p.id,
    p.name AS product_name,
    p.price,
    p.in_stock,
    c.name AS category_name
FROM public.products p
LEFT JOIN public.categories c ON c.id = p.category_id;
