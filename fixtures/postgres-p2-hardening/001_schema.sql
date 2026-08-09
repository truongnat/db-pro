-- P2 Hardening — PostgreSQL fixture schema
-- Covers: int PK, UUID PK, composite PK, no-PK table, FK (CASCADE/RESTRICT),
-- unique, nullable, boolean, text, numeric/decimal, date, timestamp, JSON/JSONB,
-- enum, varchar, bytea, array, regular/unique/composite/partial indexes,
-- views, triggers, functions, sequences, generated columns.

-- ── Enum types ─────────────────────────────────────────────────
CREATE TYPE order_status AS ENUM ('pending', 'processing', 'shipped', 'delivered', 'cancelled');
CREATE TYPE membership_role AS ENUM ('owner', 'admin', 'member', 'viewer');
CREATE TYPE audit_action AS ENUM ('create', 'update', 'delete', 'login', 'logout', 'export');

-- ── Sequence ───────────────────────────────────────────────────
CREATE SEQUENCE audit_seq START 1;

-- ══════════════════════════════════════════════════════════════
-- CORE TABLES
-- ══════════════════════════════════════════════════════════════

-- ── Organizations (int PK) ─────────────────────────────────────
CREATE TABLE organizations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    plan VARCHAR(50) NOT NULL DEFAULT 'free',
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Users (UUID PK) ────────────────────────────────────────────
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) NOT NULL,
    display_name VARCHAR(200),
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_login_at TIMESTAMPTZ,
    preferences JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Profiles (1:1 with users, FK RESTRICT) ─────────────────────
CREATE TABLE profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE RESTRICT,
    bio TEXT,
    avatar_url VARCHAR(500),
    phone VARCHAR(50),
    location VARCHAR(200),
    date_of_birth DATE,
    website VARCHAR(500),
    social_links JSONB DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Organization members (composite PK, FK CASCADE + RESTRICT) ─
CREATE TABLE organization_members (
    organization_id INTEGER NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    role membership_role NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (organization_id, user_id)
);

-- ── Categories (int PK, self-referencing FK) ───────────────────
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Products (UUID PK, numeric, JSONB, array) ──────────────────
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    sku VARCHAR(50) UNIQUE NOT NULL,
    slug VARCHAR(200) UNIQUE NOT NULL,
    description TEXT,
    price NUMERIC(12, 2) NOT NULL DEFAULT 0,
    cost NUMERIC(12, 2),
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    tags TEXT[] DEFAULT '{}',
    attributes JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    stock_quantity INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Product categories (many-to-many, composite PK) ────────────
CREATE TABLE product_categories (
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (product_id, category_id)
);

-- ── Orders (bigserial PK, FK to users) ─────────────────────────
CREATE TABLE orders (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    organization_id INTEGER REFERENCES organizations(id) ON DELETE SET NULL,
    status order_status NOT NULL DEFAULT 'pending',
    subtotal NUMERIC(12, 2) NOT NULL DEFAULT 0,
    tax NUMERIC(12, 2) NOT NULL DEFAULT 0,
    discount NUMERIC(12, 2) NOT NULL DEFAULT 0,
    total NUMERIC(12, 2) NOT NULL DEFAULT 0,
    shipping_address JSONB,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Order items (composite PK, FK CASCADE) ─────────────────────
CREATE TABLE order_items (
    order_id BIGINT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price NUMERIC(12, 2) NOT NULL,
    discount NUMERIC(12, 2) NOT NULL DEFAULT 0,
    PRIMARY KEY (order_id, product_id)
);

-- ── Settings (key-value, no PK — intentional for testing) ──────
CREATE TABLE settings (
    key VARCHAR(200) NOT NULL UNIQUE,
    value JSONB NOT NULL DEFAULT '{}',
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Audit logs (no PK, for testing no-PK tables) ───────────────
CREATE TABLE audit_logs (
    id BIGINT DEFAULT nextval('audit_seq'),
    action audit_action NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id VARCHAR(100),
    actor_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    details JSONB,
    ip_address VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Sessions (for bulk data testing) ───────────────────────────
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Empty table (for empty-state testing) ──────────────────────
CREATE TABLE empty_table (
    id SERIAL PRIMARY KEY,
    label VARCHAR(50)
);

-- ── Table with weird identifiers ───────────────────────────────
CREATE TABLE "weird""name" (
    id SERIAL PRIMARY KEY,
    "col with spaces" VARCHAR(50),
    "SELECT" VARCHAR(50)
);

-- ── Large text / blob ──────────────────────────────────────────
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    title VARCHAR(200),
    body TEXT,
    binary_data BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Employees (generated column) ───────────────────────────────
CREATE TABLE employees (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    full_name VARCHAR(101) GENERATED ALWAYS AS (first_name || ' ' || last_name) STORED,
    email VARCHAR(255) UNIQUE NOT NULL,
    hire_date DATE NOT NULL DEFAULT CURRENT_DATE,
    salary NUMERIC(10, 2),
    department VARCHAR(100)
);

-- ══════════════════════════════════════════════════════════════
-- VIEWS
-- ══════════════════════════════════════════════════════════════

CREATE VIEW active_users AS
SELECT id, email, username, display_name, created_at
FROM users
WHERE is_active = true;

CREATE VIEW order_summary AS
SELECT
    o.id AS order_id,
    u.username,
    o.status,
    o.total,
    o.created_at,
    COUNT(oi.product_id) AS item_count
FROM orders o
JOIN users u ON u.id = o.user_id
LEFT JOIN order_items oi ON oi.order_id = o.id
GROUP BY o.id, u.username, o.status, o.total, o.created_at;

CREATE VIEW organization_members_view AS
SELECT
    om.organization_id,
    org.name AS organization_name,
    om.user_id,
    u.username,
    om.role,
    om.joined_at
FROM organization_members om
JOIN organizations org ON org.id = om.organization_id
JOIN users u ON u.id = om.user_id;

-- ══════════════════════════════════════════════════════════════
-- INDEXES
-- ══════════════════════════════════════════════════════════════

-- Regular indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_products_category ON products(category_id);
CREATE INDEX idx_products_sku ON products(sku);
CREATE INDEX idx_orders_user ON orders(user_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created ON orders(created_at DESC);
CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);

-- Unique indexes
CREATE UNIQUE INDEX idx_categories_slug ON categories(slug);
CREATE UNIQUE INDEX idx_products_slug ON products(slug);

-- Composite indexes
CREATE INDEX idx_orders_user_status ON orders(user_id, status);
CREATE INDEX idx_orders_created_status ON orders(created_at DESC, status);
CREATE INDEX idx_org_members_user_role ON organization_members(user_id, role);

-- Expression index
CREATE INDEX idx_employees_lower_email ON employees(LOWER(email));
CREATE INDEX idx_users_lower_username ON users(LOWER(username));

-- Partial index (PostgreSQL-specific)
CREATE INDEX idx_orders_pending ON orders(created_at DESC) WHERE status = 'pending';
CREATE INDEX idx_products_active ON products(name) WHERE is_active = true;

-- GIN index for JSONB
CREATE INDEX idx_products_attributes ON products USING gin(attributes);
CREATE INDEX idx_users_preferences ON users USING gin(preferences);
CREATE INDEX idx_audit_logs_details ON audit_logs USING gin(details);

-- ══════════════════════════════════════════════════════════════
-- TRIGGER FUNCTIONS
-- ══════════════════════════════════════════════════════════════

CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION calculate_order_total()
RETURNS TRIGGER AS $$
BEGIN
    NEW.subtotal := (
        SELECT COALESCE(SUM(oi.quantity * oi.unit_price - oi.discount), 0)
        FROM order_items oi WHERE oi.order_id = NEW.id
    );
    NEW.total := NEW.subtotal + NEW.tax - NEW.discount;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ── Standalone function ────────────────────────────────────────
CREATE OR REPLACE FUNCTION calculate_total(p_order_id BIGINT)
RETURNS NUMERIC AS $$
BEGIN
    RETURN (SELECT COALESCE(SUM(quantity * unit_price - discount), 0)
            FROM order_items WHERE order_id = p_order_id);
END;
$$ LANGUAGE plpgsql;

-- ── Function with multiple return types ────────────────────────
CREATE OR REPLACE FUNCTION get_user_orders(p_user_id UUID, p_limit INTEGER DEFAULT 10)
RETURNS TABLE(order_id BIGINT, status order_status, total NUMERIC, created_at TIMESTAMPTZ) AS $$
BEGIN
    RETURN QUERY
    SELECT o.id, o.status, o.total, o.created_at
    FROM orders o
    WHERE o.user_id = p_user_id
    ORDER BY o.created_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- ══════════════════════════════════════════════════════════════
-- TRIGGERS
-- ══════════════════════════════════════════════════════════════

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER products_updated_at
    BEFORE UPDATE ON products
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER orders_updated_at
    BEFORE UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION update_timestamp();

CREATE TRIGGER orders_recalculate_total
    AFTER INSERT OR UPDATE ON order_items
    FOR EACH ROW EXECUTE FUNCTION calculate_order_total();
