import sys
import random
import datetime
import json
import psycopg2
from psycopg2.extras import execute_batch

def get_connection():
    return psycopg2.connect(
        dbname="enterprise_db",
        user="truongdq",
        host="localhost",
        port=5432
    )

SCHEMAS = [
    "design_system",
    "identity",
    "catalog",
    "inventory",
    "orders",
    "billing",
    "crm",
    "marketing",
    "logistics",
]

DDL_SCRIPT = """
-- Drop old schemas if exist
DROP SCHEMA IF EXISTS design_system CASCADE;
DROP SCHEMA IF EXISTS identity CASCADE;
DROP SCHEMA IF EXISTS catalog CASCADE;
DROP SCHEMA IF EXISTS inventory CASCADE;
DROP SCHEMA IF EXISTS orders CASCADE;
DROP SCHEMA IF EXISTS billing CASCADE;
DROP SCHEMA IF EXISTS crm CASCADE;
DROP SCHEMA IF EXISTS marketing CASCADE;
DROP SCHEMA IF EXISTS logistics CASCADE;

-- Create Schemas
CREATE SCHEMA design_system;
CREATE SCHEMA identity;
CREATE SCHEMA catalog;
CREATE SCHEMA inventory;
CREATE SCHEMA orders;
CREATE SCHEMA billing;
CREATE SCHEMA crm;
CREATE SCHEMA marketing;
CREATE SCHEMA logistics;

-- 1. design_system (15 tables)
CREATE TABLE design_system.tokens (
    id SERIAL PRIMARY KEY,
    name VARCHAR(120) NOT NULL UNIQUE,
    category VARCHAR(50) NOT NULL,
    value VARCHAR(255) NOT NULL,
    token_type VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE design_system.token_aliases (
    id SERIAL PRIMARY KEY,
    token_id INT NOT NULL REFERENCES design_system.tokens(id) ON DELETE CASCADE,
    alias_name VARCHAR(120) NOT NULL,
    mode VARCHAR(50) NOT NULL,
    resolved_value VARCHAR(255) NOT NULL
);

CREATE TABLE design_system.color_palettes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    hue VARCHAR(50) NOT NULL,
    hex_value VARCHAR(7) NOT NULL,
    rgb_value VARCHAR(30) NOT NULL,
    is_dark_mode BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE design_system.typography_styles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    font_family VARCHAR(80) NOT NULL,
    font_size_px NUMERIC(6, 2) NOT NULL,
    line_height_px NUMERIC(6, 2) NOT NULL,
    font_weight INT NOT NULL
);

CREATE TABLE design_system.component_definitions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    category VARCHAR(60) NOT NULL,
    status VARCHAR(30) NOT NULL,
    doc_url TEXT
);

CREATE TABLE design_system.component_variants (
    id SERIAL PRIMARY KEY,
    component_id INT NOT NULL REFERENCES design_system.component_definitions(id) ON DELETE CASCADE,
    variant_name VARCHAR(80) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    description TEXT
);

CREATE TABLE design_system.component_props (
    id SERIAL PRIMARY KEY,
    component_id INT NOT NULL REFERENCES design_system.component_definitions(id) ON DELETE CASCADE,
    prop_name VARCHAR(80) NOT NULL,
    prop_type VARCHAR(50) NOT NULL,
    default_value VARCHAR(100),
    is_required BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE design_system.icon_assets (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    category VARCHAR(50) NOT NULL,
    svg_path TEXT NOT NULL,
    viewbox VARCHAR(30) NOT NULL DEFAULT '0 0 24 24',
    tags VARCHAR(255)
);

CREATE TABLE design_system.illustration_assets (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    aspect_ratio VARCHAR(20) NOT NULL,
    file_url TEXT NOT NULL,
    format VARCHAR(20) NOT NULL,
    file_size_kb INT NOT NULL
);

CREATE TABLE design_system.animation_presets (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    duration_ms INT NOT NULL,
    easing_curve VARCHAR(60) NOT NULL,
    properties_affected VARCHAR(120) NOT NULL
);

CREATE TABLE design_system.theme_definitions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    code VARCHAR(50) NOT NULL UNIQUE,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    author VARCHAR(80) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE design_system.theme_token_overrides (
    id SERIAL PRIMARY KEY,
    theme_id INT NOT NULL REFERENCES design_system.theme_definitions(id) ON DELETE CASCADE,
    token_id INT NOT NULL REFERENCES design_system.tokens(id) ON DELETE CASCADE,
    override_value VARCHAR(255) NOT NULL
);

CREATE TABLE design_system.layout_grids (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    columns_count INT NOT NULL,
    gutter_px NUMERIC(6, 2) NOT NULL,
    margin_px NUMERIC(6, 2) NOT NULL
);

CREATE TABLE design_system.breakpoints (
    id SERIAL PRIMARY KEY,
    name VARCHAR(40) NOT NULL,
    min_width_px INT NOT NULL,
    max_width_px INT NOT NULL,
    orientation VARCHAR(30) NOT NULL
);

CREATE TABLE design_system.figma_sync_jobs (
    id SERIAL PRIMARY KEY,
    file_key VARCHAR(80) NOT NULL,
    status VARCHAR(40) NOT NULL,
    synced_components_count INT NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. identity (15 tables)
CREATE TABLE identity.tenants (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    plan VARCHAR(50) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE identity.tenant_settings (
    id SERIAL PRIMARY KEY,
    tenant_id INT NOT NULL REFERENCES identity.tenants(id) ON DELETE CASCADE,
    setting_key VARCHAR(80) NOT NULL,
    setting_value JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE identity.users (
    id SERIAL PRIMARY KEY,
    tenant_id INT NOT NULL REFERENCES identity.tenants(id) ON DELETE CASCADE,
    email VARCHAR(120) NOT NULL UNIQUE,
    full_name VARCHAR(100) NOT NULL,
    status VARCHAR(30) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE identity.user_profiles (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES identity.users(id) ON DELETE CASCADE,
    avatar_url TEXT,
    bio TEXT,
    phone VARCHAR(30),
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC'
);

CREATE TABLE identity.roles (
    id SERIAL PRIMARY KEY,
    tenant_id INT NOT NULL REFERENCES identity.tenants(id) ON DELETE CASCADE,
    name VARCHAR(80) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE identity.permissions (
    id SERIAL PRIMARY KEY,
    resource VARCHAR(80) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT
);

CREATE TABLE identity.user_roles (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES identity.users(id) ON DELETE CASCADE,
    role_id INT NOT NULL REFERENCES identity.roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE identity.role_permissions (
    id SERIAL PRIMARY KEY,
    role_id INT NOT NULL REFERENCES identity.roles(id) ON DELETE CASCADE,
    permission_id INT NOT NULL REFERENCES identity.permissions(id) ON DELETE CASCADE
);

CREATE TABLE identity.teams (
    id SERIAL PRIMARY KEY,
    tenant_id INT NOT NULL REFERENCES identity.tenants(id) ON DELETE CASCADE,
    name VARCHAR(80) NOT NULL,
    lead_user_id INT NOT NULL REFERENCES identity.users(id)
);

CREATE TABLE identity.team_members (
    id SERIAL PRIMARY KEY,
    team_id INT NOT NULL REFERENCES identity.teams(id) ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES identity.users(id) ON DELETE CASCADE,
    role_in_team VARCHAR(50) NOT NULL
);

CREATE TABLE identity.sessions (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES identity.users(id) ON DELETE CASCADE,
    session_token VARCHAR(128) NOT NULL UNIQUE,
    ip_address VARCHAR(45) NOT NULL,
    user_agent TEXT,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE identity.api_keys (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES identity.users(id) ON DELETE CASCADE,
    key_prefix VARCHAR(16) NOT NULL,
    key_hash VARCHAR(64) NOT NULL,
    permissions_scope VARCHAR(100) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE identity.oauth_providers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(60) NOT NULL,
    client_id VARCHAR(100) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE identity.oauth_accounts (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES identity.users(id) ON DELETE CASCADE,
    provider_id INT NOT NULL REFERENCES identity.oauth_providers(id) ON DELETE CASCADE,
    provider_user_id VARCHAR(100) NOT NULL
);

CREATE TABLE identity.audit_logs (
    id SERIAL PRIMARY KEY,
    tenant_id INT NOT NULL REFERENCES identity.tenants(id) ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES identity.users(id) ON DELETE CASCADE,
    action VARCHAR(80) NOT NULL,
    target_entity VARCHAR(80) NOT NULL,
    details JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. catalog (15 tables)
CREATE TABLE catalog.categories (
    id SERIAL PRIMARY KEY,
    parent_id INT REFERENCES catalog.categories(id),
    name VARCHAR(80) NOT NULL,
    slug VARCHAR(80) NOT NULL UNIQUE,
    display_order INT NOT NULL DEFAULT 0
);

CREATE TABLE catalog.brands (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    website TEXT,
    logo_url TEXT
);

CREATE TABLE catalog.products (
    id SERIAL PRIMARY KEY,
    category_id INT NOT NULL REFERENCES catalog.categories(id),
    brand_id INT NOT NULL REFERENCES catalog.brands(id),
    name VARCHAR(120) NOT NULL,
    sku VARCHAR(60) NOT NULL UNIQUE,
    base_price NUMERIC(10, 2) NOT NULL,
    status VARCHAR(30) NOT NULL DEFAULT 'published'
);

CREATE TABLE catalog.product_variants (
    id SERIAL PRIMARY KEY,
    product_id INT NOT NULL REFERENCES catalog.products(id) ON DELETE CASCADE,
    sku VARCHAR(60) NOT NULL UNIQUE,
    price NUMERIC(10, 2) NOT NULL,
    barcode VARCHAR(50),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE catalog.product_attributes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(60) NOT NULL,
    attribute_type VARCHAR(40) NOT NULL
);

CREATE TABLE catalog.attribute_values (
    id SERIAL PRIMARY KEY,
    attribute_id INT NOT NULL REFERENCES catalog.product_attributes(id) ON DELETE CASCADE,
    value VARCHAR(60) NOT NULL,
    label VARCHAR(80) NOT NULL
);

CREATE TABLE catalog.variant_attribute_values (
    id SERIAL PRIMARY KEY,
    variant_id INT NOT NULL REFERENCES catalog.product_variants(id) ON DELETE CASCADE,
    attribute_value_id INT NOT NULL REFERENCES catalog.attribute_values(id) ON DELETE CASCADE
);

CREATE TABLE catalog.product_images (
    id SERIAL PRIMARY KEY,
    product_id INT NOT NULL REFERENCES catalog.products(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    alt_text VARCHAR(120),
    sort_order INT NOT NULL DEFAULT 0,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE catalog.tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    slug VARCHAR(50) NOT NULL UNIQUE
);

CREATE TABLE catalog.product_tags (
    id SERIAL PRIMARY KEY,
    product_id INT NOT NULL REFERENCES catalog.products(id) ON DELETE CASCADE,
    tag_id INT NOT NULL REFERENCES catalog.tags(id) ON DELETE CASCADE
);

CREATE TABLE catalog.price_lists (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    valid_from TIMESTAMPTZ NOT NULL,
    valid_to TIMESTAMPTZ NOT NULL
);

CREATE TABLE catalog.price_list_items (
    id SERIAL PRIMARY KEY,
    price_list_id INT NOT NULL REFERENCES catalog.price_lists(id) ON DELETE CASCADE,
    variant_id INT NOT NULL REFERENCES catalog.product_variants(id) ON DELETE CASCADE,
    price NUMERIC(10, 2) NOT NULL
);

CREATE TABLE catalog.product_reviews (
    id SERIAL PRIMARY KEY,
    product_id INT NOT NULL REFERENCES catalog.products(id) ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES identity.users(id),
    rating INT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment TEXT,
    is_verified BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE catalog.review_votes (
    id SERIAL PRIMARY KEY,
    review_id INT NOT NULL REFERENCES catalog.product_reviews(id) ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES identity.users(id),
    is_helpful BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE catalog.product_bundles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    discount_percent NUMERIC(5, 2) NOT NULL DEFAULT 10.0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- 4. inventory (12 tables)
CREATE TABLE inventory.warehouses (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    code VARCHAR(30) NOT NULL UNIQUE,
    address TEXT NOT NULL,
    city VARCHAR(60) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE inventory.warehouse_zones (
    id SERIAL PRIMARY KEY,
    warehouse_id INT NOT NULL REFERENCES inventory.warehouses(id) ON DELETE CASCADE,
    zone_code VARCHAR(30) NOT NULL,
    zone_type VARCHAR(40) NOT NULL
);

CREATE TABLE inventory.locations (
    id SERIAL PRIMARY KEY,
    zone_id INT NOT NULL REFERENCES inventory.warehouse_zones(id) ON DELETE CASCADE,
    aisle VARCHAR(20) NOT NULL,
    rack VARCHAR(20) NOT NULL,
    shelf VARCHAR(20) NOT NULL,
    bin VARCHAR(20) NOT NULL
);

CREATE TABLE inventory.suppliers (
    id SERIAL PRIMARY KEY,
    company_name VARCHAR(120) NOT NULL,
    contact_name VARCHAR(80) NOT NULL,
    email VARCHAR(100) NOT NULL,
    phone VARCHAR(30) NOT NULL
);

CREATE TABLE inventory.supplier_products (
    id SERIAL PRIMARY KEY,
    supplier_id INT NOT NULL REFERENCES inventory.suppliers(id) ON DELETE CASCADE,
    product_id INT NOT NULL REFERENCES catalog.products(id) ON DELETE CASCADE,
    supplier_sku VARCHAR(60) NOT NULL,
    cost_price NUMERIC(10, 2) NOT NULL
);

CREATE TABLE inventory.stock_levels (
    id SERIAL PRIMARY KEY,
    warehouse_id INT NOT NULL REFERENCES inventory.warehouses(id) ON DELETE CASCADE,
    variant_id INT NOT NULL REFERENCES catalog.product_variants(id) ON DELETE CASCADE,
    quantity_on_hand INT NOT NULL DEFAULT 0,
    quantity_reserved INT NOT NULL DEFAULT 0
);

CREATE TABLE inventory.stock_adjustments (
    id SERIAL PRIMARY KEY,
    warehouse_id INT NOT NULL REFERENCES inventory.warehouses(id) ON DELETE CASCADE,
    reason VARCHAR(80) NOT NULL,
    adjusted_by_user_id INT NOT NULL REFERENCES identity.users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE inventory.stock_adjustment_items (
    id SERIAL PRIMARY KEY,
    adjustment_id INT NOT NULL REFERENCES inventory.stock_adjustments(id) ON DELETE CASCADE,
    variant_id INT NOT NULL REFERENCES catalog.product_variants(id) ON DELETE CASCADE,
    quantity_delta INT NOT NULL
);

CREATE TABLE inventory.purchase_orders (
    id SERIAL PRIMARY KEY,
    supplier_id INT NOT NULL REFERENCES inventory.suppliers(id) ON DELETE CASCADE,
    warehouse_id INT NOT NULL REFERENCES inventory.warehouses(id) ON DELETE CASCADE,
    po_number VARCHAR(50) NOT NULL UNIQUE,
    status VARCHAR(30) NOT NULL,
    total_cost NUMERIC(12, 2) NOT NULL
);

CREATE TABLE inventory.purchase_order_items (
    id SERIAL PRIMARY KEY,
    purchase_order_id INT NOT NULL REFERENCES inventory.purchase_orders(id) ON DELETE CASCADE,
    variant_id INT NOT NULL REFERENCES catalog.product_variants(id) ON DELETE CASCADE,
    quantity_ordered INT NOT NULL,
    unit_cost NUMERIC(10, 2) NOT NULL
);

CREATE TABLE inventory.stock_transfers (
    id SERIAL PRIMARY KEY,
    from_warehouse_id INT NOT NULL REFERENCES inventory.warehouses(id),
    to_warehouse_id INT NOT NULL REFERENCES inventory.warehouses(id),
    status VARCHAR(30) NOT NULL,
    initiated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE inventory.stock_transfer_items (
    id SERIAL PRIMARY KEY,
    transfer_id INT NOT NULL REFERENCES inventory.stock_transfers(id) ON DELETE CASCADE,
    variant_id INT NOT NULL REFERENCES catalog.product_variants(id) ON DELETE CASCADE,
    quantity INT NOT NULL
);

-- 5. orders (14 tables)
CREATE TABLE orders.customers (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(60) NOT NULL,
    last_name VARCHAR(60) NOT NULL,
    email VARCHAR(120) NOT NULL UNIQUE,
    phone VARCHAR(30),
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE orders.customer_addresses (
    id SERIAL PRIMARY KEY,
    customer_id INT NOT NULL REFERENCES orders.customers(id) ON DELETE CASCADE,
    address_type VARCHAR(20) NOT NULL,
    street TEXT NOT NULL,
    city VARCHAR(60) NOT NULL,
    postal_code VARCHAR(20) NOT NULL,
    country VARCHAR(50) NOT NULL DEFAULT 'USA'
);

CREATE TABLE orders.customer_groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR(60) NOT NULL,
    discount_rate NUMERIC(5, 2) NOT NULL DEFAULT 0.0
);

CREATE TABLE orders.customer_group_members (
    id SERIAL PRIMARY KEY,
    customer_id INT NOT NULL REFERENCES orders.customers(id) ON DELETE CASCADE,
    group_id INT NOT NULL REFERENCES orders.customer_groups(id) ON DELETE CASCADE
);

CREATE TABLE orders.carts (
    id SERIAL PRIMARY KEY,
    customer_id INT NOT NULL REFERENCES orders.customers(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE orders.cart_items (
    id SERIAL PRIMARY KEY,
    cart_id INT NOT NULL REFERENCES orders.carts(id) ON DELETE CASCADE,
    variant_id INT NOT NULL REFERENCES catalog.product_variants(id) ON DELETE CASCADE,
    quantity INT NOT NULL,
    unit_price NUMERIC(10, 2) NOT NULL
);

CREATE TABLE orders.orders (
    id SERIAL PRIMARY KEY,
    customer_id INT NOT NULL REFERENCES orders.customers(id) ON DELETE CASCADE,
    order_number VARCHAR(50) NOT NULL UNIQUE,
    order_status VARCHAR(30) NOT NULL,
    subtotal NUMERIC(10, 2) NOT NULL,
    tax_total NUMERIC(10, 2) NOT NULL,
    total_amount NUMERIC(10, 2) NOT NULL,
    ordered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE orders.order_items (
    id SERIAL PRIMARY KEY,
    order_id INT NOT NULL REFERENCES orders.orders(id) ON DELETE CASCADE,
    variant_id INT NOT NULL REFERENCES catalog.product_variants(id) ON DELETE CASCADE,
    product_name VARCHAR(120) NOT NULL,
    quantity INT NOT NULL,
    unit_price NUMERIC(10, 2) NOT NULL,
    total_price NUMERIC(10, 2) NOT NULL
);

CREATE TABLE orders.order_status_history (
    id SERIAL PRIMARY KEY,
    order_id INT NOT NULL REFERENCES orders.orders(id) ON DELETE CASCADE,
    old_status VARCHAR(30) NOT NULL,
    new_status VARCHAR(30) NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE orders.order_discounts (
    id SERIAL PRIMARY KEY,
    order_id INT NOT NULL REFERENCES orders.orders(id) ON DELETE CASCADE,
    code VARCHAR(40) NOT NULL,
    amount NUMERIC(10, 2) NOT NULL
);

CREATE TABLE orders.shipments (
    id SERIAL PRIMARY KEY,
    order_id INT NOT NULL REFERENCES orders.orders(id) ON DELETE CASCADE,
    tracking_number VARCHAR(60) NOT NULL UNIQUE,
    carrier_code VARCHAR(30) NOT NULL,
    status VARCHAR(30) NOT NULL,
    shipped_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE orders.shipment_items (
    id SERIAL PRIMARY KEY,
    shipment_id INT NOT NULL REFERENCES orders.shipments(id) ON DELETE CASCADE,
    order_item_id INT NOT NULL REFERENCES orders.order_items(id) ON DELETE CASCADE,
    quantity INT NOT NULL
);

CREATE TABLE orders.returns (
    id SERIAL PRIMARY KEY,
    order_id INT NOT NULL REFERENCES orders.orders(id) ON DELETE CASCADE,
    return_number VARCHAR(50) NOT NULL UNIQUE,
    status VARCHAR(30) NOT NULL,
    reason TEXT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE orders.return_items (
    id SERIAL PRIMARY KEY,
    return_id INT NOT NULL REFERENCES orders.returns(id) ON DELETE CASCADE,
    order_item_id INT NOT NULL REFERENCES orders.order_items(id) ON DELETE CASCADE,
    quantity INT NOT NULL,
    condition VARCHAR(40) NOT NULL
);

-- 6. billing (12 tables)
CREATE TABLE billing.payment_methods (
    id SERIAL PRIMARY KEY,
    customer_id INT NOT NULL REFERENCES orders.customers(id) ON DELETE CASCADE,
    method_type VARCHAR(30) NOT NULL,
    provider VARCHAR(40) NOT NULL,
    masked_card VARCHAR(20) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE billing.invoices (
    id SERIAL PRIMARY KEY,
    order_id INT NOT NULL REFERENCES orders.orders(id) ON DELETE CASCADE,
    invoice_number VARCHAR(50) NOT NULL UNIQUE,
    status VARCHAR(30) NOT NULL,
    subtotal NUMERIC(10, 2) NOT NULL,
    tax NUMERIC(10, 2) NOT NULL,
    total NUMERIC(10, 2) NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing.invoice_items (
    id SERIAL PRIMARY KEY,
    invoice_id INT NOT NULL REFERENCES billing.invoices(id) ON DELETE CASCADE,
    description VARCHAR(150) NOT NULL,
    quantity INT NOT NULL,
    unit_price NUMERIC(10, 2) NOT NULL,
    amount NUMERIC(10, 2) NOT NULL
);

CREATE TABLE billing.payment_transactions (
    id SERIAL PRIMARY KEY,
    invoice_id INT NOT NULL REFERENCES billing.invoices(id) ON DELETE CASCADE,
    payment_method_id INT NOT NULL REFERENCES billing.payment_methods(id),
    transaction_id VARCHAR(80) NOT NULL UNIQUE,
    amount NUMERIC(10, 2) NOT NULL,
    status VARCHAR(30) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing.refunds (
    id SERIAL PRIMARY KEY,
    transaction_id INT NOT NULL REFERENCES billing.payment_transactions(id) ON DELETE CASCADE,
    refund_reference VARCHAR(80) NOT NULL UNIQUE,
    amount NUMERIC(10, 2) NOT NULL,
    reason VARCHAR(80) NOT NULL,
    refunded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing.tax_jurisdictions (
    id SERIAL PRIMARY KEY,
    country_code VARCHAR(2) NOT NULL,
    state_code VARCHAR(10) NOT NULL,
    jurisdiction_name VARCHAR(80) NOT NULL
);

CREATE TABLE billing.tax_rates (
    id SERIAL PRIMARY KEY,
    jurisdiction_id INT NOT NULL REFERENCES billing.tax_jurisdictions(id) ON DELETE CASCADE,
    tax_name VARCHAR(60) NOT NULL,
    rate_percent NUMERIC(5, 2) NOT NULL
);

CREATE TABLE billing.subscriptions (
    id SERIAL PRIMARY KEY,
    customer_id INT NOT NULL REFERENCES orders.customers(id) ON DELETE CASCADE,
    plan_name VARCHAR(60) NOT NULL,
    status VARCHAR(30) NOT NULL,
    interval VARCHAR(20) NOT NULL,
    current_period_start TIMESTAMPTZ NOT NULL,
    current_period_end TIMESTAMPTZ NOT NULL
);

CREATE TABLE billing.subscription_items (
    id SERIAL PRIMARY KEY,
    subscription_id INT NOT NULL REFERENCES billing.subscriptions(id) ON DELETE CASCADE,
    item_name VARCHAR(80) NOT NULL,
    quantity INT NOT NULL,
    unit_amount NUMERIC(10, 2) NOT NULL
);

CREATE TABLE billing.payment_gateways (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    is_test_mode BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE billing.gateway_logs (
    id SERIAL PRIMARY KEY,
    gateway_id INT NOT NULL REFERENCES billing.payment_gateways(id) ON DELETE CASCADE,
    event_type VARCHAR(60) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing.credit_notes (
    id SERIAL PRIMARY KEY,
    invoice_id INT NOT NULL REFERENCES billing.invoices(id) ON DELETE CASCADE,
    credit_number VARCHAR(50) NOT NULL UNIQUE,
    total_credit NUMERIC(10, 2) NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 7. crm (12 tables)
CREATE TABLE crm.leads (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(60) NOT NULL,
    last_name VARCHAR(60) NOT NULL,
    email VARCHAR(120) NOT NULL,
    company VARCHAR(80),
    status VARCHAR(30) NOT NULL,
    source VARCHAR(40) NOT NULL
);

CREATE TABLE crm.contacts (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(60) NOT NULL,
    last_name VARCHAR(60) NOT NULL,
    email VARCHAR(120) NOT NULL,
    phone VARCHAR(30),
    job_title VARCHAR(80)
);

CREATE TABLE crm.accounts (
    id SERIAL PRIMARY KEY,
    account_name VARCHAR(100) NOT NULL,
    industry VARCHAR(60) NOT NULL,
    annual_revenue NUMERIC(14, 2) NOT NULL DEFAULT 0.0,
    website TEXT
);

CREATE TABLE crm.pipelines (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE crm.deal_stages (
    id SERIAL PRIMARY KEY,
    pipeline_id INT NOT NULL REFERENCES crm.pipelines(id) ON DELETE CASCADE,
    stage_name VARCHAR(60) NOT NULL,
    probability_percent INT NOT NULL,
    sort_order INT NOT NULL
);

CREATE TABLE crm.deals (
    id SERIAL PRIMARY KEY,
    account_id INT NOT NULL REFERENCES crm.accounts(id) ON DELETE CASCADE,
    stage_id INT NOT NULL REFERENCES crm.deal_stages(id) ON DELETE CASCADE,
    deal_name VARCHAR(120) NOT NULL,
    expected_amount NUMERIC(12, 2) NOT NULL,
    close_date TIMESTAMPTZ NOT NULL
);

CREATE TABLE crm.deal_participants (
    id SERIAL PRIMARY KEY,
    deal_id INT NOT NULL REFERENCES crm.deals(id) ON DELETE CASCADE,
    contact_id INT NOT NULL REFERENCES crm.contacts(id) ON DELETE CASCADE,
    role_in_deal VARCHAR(50) NOT NULL
);

CREATE TABLE crm.activities (
    id SERIAL PRIMARY KEY,
    activity_type VARCHAR(40) NOT NULL,
    subject VARCHAR(150) NOT NULL,
    due_date TIMESTAMPTZ NOT NULL,
    is_completed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE crm.activity_relations (
    id SERIAL PRIMARY KEY,
    activity_id INT NOT NULL REFERENCES crm.activities(id) ON DELETE CASCADE,
    entity_type VARCHAR(40) NOT NULL,
    entity_id INT NOT NULL
);

CREATE TABLE crm.support_tickets (
    id SERIAL PRIMARY KEY,
    ticket_code VARCHAR(40) NOT NULL UNIQUE,
    customer_id INT NOT NULL REFERENCES orders.customers(id),
    subject VARCHAR(150) NOT NULL,
    priority VARCHAR(20) NOT NULL,
    status VARCHAR(30) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE crm.ticket_messages (
    id SERIAL PRIMARY KEY,
    ticket_id INT NOT NULL REFERENCES crm.support_tickets(id) ON DELETE CASCADE,
    sender_type VARCHAR(20) NOT NULL,
    sender_id INT NOT NULL,
    body TEXT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE crm.ticket_tags (
    id SERIAL PRIMARY KEY,
    ticket_id INT NOT NULL REFERENCES crm.support_tickets(id) ON DELETE CASCADE,
    tag_name VARCHAR(40) NOT NULL
);

-- 8. marketing (10 tables)
CREATE TABLE marketing.campaigns (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    campaign_type VARCHAR(40) NOT NULL,
    budget NUMERIC(10, 2) NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ NOT NULL
);

CREATE TABLE marketing.campaign_channels (
    id SERIAL PRIMARY KEY,
    campaign_id INT NOT NULL REFERENCES marketing.campaigns(id) ON DELETE CASCADE,
    channel_name VARCHAR(50) NOT NULL,
    spend NUMERIC(10, 2) NOT NULL
);

CREATE TABLE marketing.email_templates (
    id SERIAL PRIMARY KEY,
    template_name VARCHAR(80) NOT NULL,
    subject_line VARCHAR(150) NOT NULL,
    body_html TEXT NOT NULL
);

CREATE TABLE marketing.segments (
    id SERIAL PRIMARY KEY,
    segment_name VARCHAR(80) NOT NULL,
    filter_criteria JSONB NOT NULL,
    members_count INT NOT NULL DEFAULT 0
);

CREATE TABLE marketing.newsletters (
    id SERIAL PRIMARY KEY,
    campaign_id INT NOT NULL REFERENCES marketing.campaigns(id) ON DELETE CASCADE,
    template_id INT NOT NULL REFERENCES marketing.email_templates(id) ON DELETE CASCADE,
    title VARCHAR(120) NOT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE marketing.subscribers (
    id SERIAL PRIMARY KEY,
    email VARCHAR(120) NOT NULL UNIQUE,
    is_subscribed BOOLEAN NOT NULL DEFAULT TRUE,
    opted_in_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE marketing.segment_members (
    id SERIAL PRIMARY KEY,
    segment_id INT NOT NULL REFERENCES marketing.segments(id) ON DELETE CASCADE,
    subscriber_id INT NOT NULL REFERENCES marketing.subscribers(id) ON DELETE CASCADE
);

CREATE TABLE marketing.coupons (
    id SERIAL PRIMARY KEY,
    code VARCHAR(40) NOT NULL UNIQUE,
    discount_type VARCHAR(20) NOT NULL,
    value NUMERIC(10, 2) NOT NULL,
    max_uses INT NOT NULL DEFAULT 1000,
    valid_until TIMESTAMPTZ NOT NULL
);

CREATE TABLE marketing.coupon_redemptions (
    id SERIAL PRIMARY KEY,
    coupon_id INT NOT NULL REFERENCES marketing.coupons(id) ON DELETE CASCADE,
    customer_id INT NOT NULL REFERENCES orders.customers(id) ON DELETE CASCADE,
    redeemed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE marketing.affiliates (
    id SERIAL PRIMARY KEY,
    partner_name VARCHAR(80) NOT NULL,
    referral_code VARCHAR(40) NOT NULL UNIQUE,
    commission_rate NUMERIC(5, 2) NOT NULL DEFAULT 5.0
);

-- 9. logistics (10 tables)
CREATE TABLE logistics.carriers (
    id SERIAL PRIMARY KEY,
    carrier_name VARCHAR(80) NOT NULL,
    api_endpoint TEXT,
    tracking_url_template TEXT
);

CREATE TABLE logistics.service_levels (
    id SERIAL PRIMARY KEY,
    carrier_id INT NOT NULL REFERENCES logistics.carriers(id) ON DELETE CASCADE,
    service_code VARCHAR(40) NOT NULL,
    service_name VARCHAR(80) NOT NULL,
    max_transit_days INT NOT NULL
);

CREATE TABLE logistics.shipping_zones (
    id SERIAL PRIMARY KEY,
    zone_name VARCHAR(60) NOT NULL,
    base_cost NUMERIC(8, 2) NOT NULL
);

CREATE TABLE logistics.zone_postal_codes (
    id SERIAL PRIMARY KEY,
    zone_id INT NOT NULL REFERENCES logistics.shipping_zones(id) ON DELETE CASCADE,
    postal_prefix VARCHAR(10) NOT NULL
);

CREATE TABLE logistics.shipping_rate_tables (
    id SERIAL PRIMARY KEY,
    service_id INT NOT NULL REFERENCES logistics.service_levels(id) ON DELETE CASCADE,
    zone_id INT NOT NULL REFERENCES logistics.shipping_zones(id) ON DELETE CASCADE,
    weight_kg NUMERIC(6, 2) NOT NULL,
    rate NUMERIC(8, 2) NOT NULL
);

CREATE TABLE logistics.vehicles (
    id SERIAL PRIMARY KEY,
    license_plate VARCHAR(30) NOT NULL UNIQUE,
    vehicle_type VARCHAR(40) NOT NULL,
    max_volume_m3 NUMERIC(6, 2) NOT NULL,
    max_weight_kg NUMERIC(8, 2) NOT NULL
);

CREATE TABLE logistics.drivers (
    id SERIAL PRIMARY KEY,
    driver_name VARCHAR(80) NOT NULL,
    license_number VARCHAR(40) NOT NULL UNIQUE,
    phone VARCHAR(30) NOT NULL,
    is_available BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE logistics.delivery_routes (
    id SERIAL PRIMARY KEY,
    vehicle_id INT NOT NULL REFERENCES logistics.vehicles(id),
    driver_id INT NOT NULL REFERENCES logistics.drivers(id),
    route_date DATE NOT NULL,
    status VARCHAR(30) NOT NULL
);

CREATE TABLE logistics.delivery_stops (
    id SERIAL PRIMARY KEY,
    route_id INT NOT NULL REFERENCES logistics.delivery_routes(id) ON DELETE CASCADE,
    stop_sequence INT NOT NULL,
    address TEXT NOT NULL,
    recipient_name VARCHAR(80) NOT NULL,
    delivery_status VARCHAR(30) NOT NULL
);

CREATE TABLE logistics.tracking_events (
    id SERIAL PRIMARY KEY,
    tracking_number VARCHAR(60) NOT NULL,
    event_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    location VARCHAR(80) NOT NULL,
    description VARCHAR(150) NOT NULL,
    status_code VARCHAR(30) NOT NULL
);
"""

def create_schema(cursor):
    print("Executing DDL to create 9 schemas and 115 tables...")
    cursor.execute(DDL_SCRIPT)
    print("DDL execution completed successfully.")

def seed_data(cursor):
    print("Seeding 100 records into EACH table in topological order...")
    N = 100
    now = datetime.datetime.now(datetime.timezone.utc)

    # 1. design_system
    # tokens
    token_rows = [
        (f"color.semantic.primary.{i}", random.choice(["color", "spacing", "typography", "elevation"]), f"#{random.randint(0x100000, 0xffffff):06x}", "color", f"Primary semantic token {i}", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.tokens (name, category, value, token_type, description, created_at) VALUES (%s, %s, %s, %s, %s, %s)", token_rows)

    # token_aliases
    alias_rows = [
        (i, f"brand.accent.{i}", "dark" if i % 2 == 0 else "light", f"#{random.randint(0x100000, 0xffffff):06x}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.token_aliases (token_id, alias_name, mode, resolved_value) VALUES (%s, %s, %s, %s)", alias_rows)

    # color_palettes
    palette_rows = [
        (f"Palette {i}", random.choice(["Blue", "Slate", "Emerald", "Amber", "Rose", "Violet"]), f"#{random.randint(0x100000, 0xffffff):06x}", f"rgb({i*2%255}, {i*3%255}, {i*5%255})", i % 2 == 0)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.color_palettes (name, hue, hex_value, rgb_value, is_dark_mode) VALUES (%s, %s, %s, %s, %s)", palette_rows)

    # typography_styles
    typo_rows = [
        (f"Typography {i}", random.choice(["Inter", "Geist", "JetBrains Mono", "SF Pro", "Fira Code"]), 10.0 + (i % 20), 14.0 + (i % 20), random.choice([400, 500, 600, 700]))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.typography_styles (name, font_family, font_size_px, line_height_px, font_weight) VALUES (%s, %s, %s, %s, %s)", typo_rows)

    # component_definitions
    comp_rows = [
        (f"Component {i}", f"component-{i}", random.choice(["Layout", "Input", "Data Display", "Feedback", "Navigation"]), random.choice(["stable", "beta", "deprecated"]), f"https://design.dbpro.dev/components/c-{i}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.component_definitions (name, slug, category, status, doc_url) VALUES (%s, %s, %s, %s, %s)", comp_rows)

    # component_variants
    var_rows = [
        (i, f"Variant-{i}", i % 5 == 0, f"Variant description for component {i}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.component_variants (component_id, variant_name, is_default, description) VALUES (%s, %s, %s, %s)", var_rows)

    # component_props
    prop_rows = [
        (i, f"prop_{i}", random.choice(["string", "boolean", "number", "callback", "node"]), "default", i % 3 == 0)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.component_props (component_id, prop_name, prop_type, default_value, is_required) VALUES (%s, %s, %s, %s, %s)", prop_rows)

    # icon_assets
    icon_rows = [
        (f"icon-{i}", random.choice(["navigation", "action", "status", "database", "file"]), f"M{i} {i} L{i+10} {i+10} Z", "0 0 24 24", f"tag{i},system,icon")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.icon_assets (name, category, svg_path, viewbox, tags) VALUES (%s, %s, %s, %s, %s)", icon_rows)

    # illustration_assets
    illus_rows = [
        (f"illustration-{i}", "16:9" if i % 2 == 0 else "4:3", f"https://assets.dbpro.dev/illus/ill_{i}.svg", "svg", 12 + (i % 50))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.illustration_assets (name, aspect_ratio, file_url, format, file_size_kb) VALUES (%s, %s, %s, %s, %s)", illus_rows)

    # animation_presets
    anim_rows = [
        (f"preset-{i}", 150 + i * 5, "cubic-bezier(0.16, 1, 0.3, 1)", "transform, opacity")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.animation_presets (name, duration_ms, easing_curve, properties_affected) VALUES (%s, %s, %s, %s)", anim_rows)

    # theme_definitions
    theme_rows = [
        (f"Theme {i}", f"theme_code_{i}", i <= 2, f"Author {i}", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.theme_definitions (name, code, is_system, author, created_at) VALUES (%s, %s, %s, %s, %s)", theme_rows)

    # theme_token_overrides
    override_rows = [
        (i, i, f"#{random.randint(0x100000, 0xffffff):06x}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.theme_token_overrides (theme_id, token_id, override_value) VALUES (%s, %s, %s)", override_rows)

    # layout_grids
    grid_rows = [
        (f"Grid-{i}", 12 if i % 2 == 0 else 16, 16.0, 24.0)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.layout_grids (name, columns_count, gutter_px, margin_px) VALUES (%s, %s, %s, %s)", grid_rows)

    # breakpoints
    bp_rows = [
        (f"bp-{i}", 320 + i * 10, 320 + i * 15, "horizontal" if i % 2 == 0 else "vertical")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.breakpoints (name, min_width_px, max_width_px, orientation) VALUES (%s, %s, %s, %s)", bp_rows)

    # figma_sync_jobs
    job_rows = [
        (f"figma_key_{i}", random.choice(["completed", "running", "failed"]), random.randint(10, 250), now - datetime.timedelta(hours=i))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO design_system.figma_sync_jobs (file_key, status, synced_components_count, synced_at) VALUES (%s, %s, %s, %s)", job_rows)

    # 2. identity
    tenant_rows = [
        (f"Tenant Enterprise {i}", f"tenant-slug-{i}", random.choice(["free", "starter", "pro", "enterprise"]), True, now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.tenants (name, slug, plan, is_active, created_at) VALUES (%s, %s, %s, %s, %s)", tenant_rows)

    t_setting_rows = [
        (i, f"auth.mfa_enforced_{i}", json.dumps({"enabled": i % 2 == 0, "grace_period_days": 14}), now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.tenant_settings (tenant_id, setting_key, setting_value, updated_at) VALUES (%s, %s, %s, %s)", t_setting_rows)

    user_rows = [
        (i, f"user{i}@enterprise{i}.com", f"Firstname Lastname {i}", "active", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.users (tenant_id, email, full_name, status, created_at) VALUES (%s, %s, %s, %s, %s)", user_rows)

    profile_rows = [
        (i, f"https://avatars.dbpro.dev/{i}.png", f"Bio of user {i}", f"+1-555-010{i:02d}", "UTC")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.user_profiles (user_id, avatar_url, bio, phone, timezone) VALUES (%s, %s, %s, %s, %s)", profile_rows)

    role_rows = [
        (i, f"Role {i}", f"Role description {i}", i <= 5)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.roles (tenant_id, name, description, is_system) VALUES (%s, %s, %s, %s)", role_rows)

    perm_rows = [
        (f"resource_{i}", random.choice(["create", "read", "update", "delete", "export"]), f"Permission to operate resource {i}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.permissions (resource, action, description) VALUES (%s, %s, %s)", perm_rows)

    user_role_rows = [
        (i, i, now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.user_roles (user_id, role_id, assigned_at) VALUES (%s, %s, %s)", user_role_rows)

    role_perm_rows = [
        (i, i)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.role_permissions (role_id, permission_id) VALUES (%s, %s)", role_perm_rows)

    team_rows = [
        (i, f"Engineering Team {i}", i)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.teams (tenant_id, name, lead_user_id) VALUES (%s, %s, %s)", team_rows)

    team_member_rows = [
        (i, i, random.choice(["member", "lead", "viewer"]))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.team_members (team_id, user_id, role_in_team) VALUES (%s, %s, %s)", team_member_rows)

    session_rows = [
        (i, f"token_hash_abc_{i}", f"192.168.1.{i%255}", "Mozilla/5.0 (Macintosh; Apple Silicon)", now + datetime.timedelta(days=7))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.sessions (user_id, session_token, ip_address, user_agent, expires_at) VALUES (%s, %s, %s, %s, %s)", session_rows)

    api_key_rows = [
        (i, f"dbp_live_{i}", f"hash_secret_key_{i}", "read,write,export", now + datetime.timedelta(days=365))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.api_keys (user_id, key_prefix, key_hash, permissions_scope, expires_at) VALUES (%s, %s, %s, %s, %s)", api_key_rows)

    provider_rows = [
        (f"OAuth Provider {i}", f"client-id-{i}", True)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.oauth_providers (name, client_id, is_enabled) VALUES (%s, %s, %s)", provider_rows)

    oauth_acc_rows = [
        (i, i, f"ext_uid_{i}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.oauth_accounts (user_id, provider_id, provider_user_id) VALUES (%s, %s, %s)", oauth_acc_rows)

    audit_rows = [
        (i, i, random.choice(["login", "update_schema", "query_exec", "export", "role_assign"]), f"entity_{i}", json.dumps({"diff": f"audit info {i}"}), now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO identity.audit_logs (tenant_id, user_id, action, target_entity, details, created_at) VALUES (%s, %s, %s, %s, %s, %s)", audit_rows)

    # 3. catalog
    cat_rows = [
        (None, f"Category {i}", f"cat-slug-{i}", i)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.categories (parent_id, name, slug, display_order) VALUES (%s, %s, %s, %s)", cat_rows)

    brand_rows = [
        (f"Brand {i}", f"https://brand{i}.com", f"https://brand{i}.com/logo.png")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.brands (name, website, logo_url) VALUES (%s, %s, %s)", brand_rows)

    prod_rows = [
        (i, i, f"Product {i}", f"SKU-PROD-{i:05d}", round(19.99 + i * 2.5, 2), "published")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.products (category_id, brand_id, name, sku, base_price, status) VALUES (%s, %s, %s, %s, %s, %s)", prod_rows)

    variant_rows = [
        (i, f"SKU-VAR-{i:05d}", round(24.99 + i * 2.5, 2), f"8901234{i:05d}", True)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.product_variants (product_id, sku, price, barcode, is_active) VALUES (%s, %s, %s, %s, %s)", variant_rows)

    attr_rows = [
        (f"Attribute {i}", random.choice(["color", "size", "material", "weight", "capacity"]))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.product_attributes (name, attribute_type) VALUES (%s, %s)", attr_rows)

    attr_val_rows = [
        (i, f"Val-{i}", f"Label-{i}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.attribute_values (attribute_id, value, label) VALUES (%s, %s, %s)", attr_val_rows)

    var_attr_rows = [
        (i, i)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.variant_attribute_values (variant_id, attribute_value_id) VALUES (%s, %s)", var_attr_rows)

    img_rows = [
        (i, f"https://images.dbpro.dev/prod_{i}.webp", f"Product image {i}", 0, True)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.product_images (product_id, url, alt_text, sort_order, is_primary) VALUES (%s, %s, %s, %s, %s)", img_rows)

    tag_rows = [
        (f"Tag-{i}", f"tag-slug-{i}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.tags (name, slug) VALUES (%s, %s)", tag_rows)

    prod_tag_rows = [
        (i, i)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.product_tags (product_id, tag_id) VALUES (%s, %s)", prod_tag_rows)

    plist_rows = [
        (f"Price List {i}", "USD", now, now + datetime.timedelta(days=180))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.price_lists (name, currency, valid_from, valid_to) VALUES (%s, %s, %s, %s)", plist_rows)

    plist_item_rows = [
        (i, i, round(29.99 + i, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.price_list_items (price_list_id, variant_id, price) VALUES (%s, %s, %s)", plist_item_rows)

    review_rows = [
        (i, i, (i % 5) + 1, f"Review comment text for product {i}", True, now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.product_reviews (product_id, user_id, rating, comment, is_verified, created_at) VALUES (%s, %s, %s, %s, %s, %s)", review_rows)

    review_vote_rows = [
        (i, i, i % 2 == 0)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.review_votes (review_id, user_id, is_helpful) VALUES (%s, %s, %s)", review_vote_rows)

    bundle_rows = [
        (f"Bundle Package {i}", 15.0, True)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO catalog.product_bundles (name, discount_percent, is_active) VALUES (%s, %s, %s)", bundle_rows)

    # 4. inventory
    wh_rows = [
        (f"Warehouse {i}", f"WH-{i:03d}", f"{i} Industrial Way", f"City {i}", True)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.warehouses (name, code, address, city, is_active) VALUES (%s, %s, %s, %s, %s)", wh_rows)

    zone_rows = [
        (i, f"Z-{i:02d}", random.choice(["ambient", "cold_storage", "hazardous", "pallet_racks"]))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.warehouse_zones (warehouse_id, zone_code, zone_type) VALUES (%s, %s, %s)", zone_rows)

    loc_rows = [
        (i, f"A{i%10}", f"R{i%5}", f"S{i%4}", f"B{i%8}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.locations (zone_id, aisle, rack, shelf, bin) VALUES (%s, %s, %s, %s, %s)", loc_rows)

    sup_rows = [
        (f"Supplier Corp {i}", f"Contact Person {i}", f"supplier{i}@supplies.com", f"+1-800-SUP-{i:03d}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.suppliers (company_name, contact_name, email, phone) VALUES (%s, %s, %s, %s)", sup_rows)

    sup_prod_rows = [
        (i, i, f"SUP-SKU-{i:05d}", round(10.0 + i, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.supplier_products (supplier_id, product_id, supplier_sku, cost_price) VALUES (%s, %s, %s, %s)", sup_prod_rows)

    stock_rows = [
        (i, i, random.randint(50, 1000), random.randint(0, 20))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.stock_levels (warehouse_id, variant_id, quantity_on_hand, quantity_reserved) VALUES (%s, %s, %s, %s)", stock_rows)

    adj_rows = [
        (i, random.choice(["cycle_count", "damaged_goods", "reconciliation", "customer_return"]), i, now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.stock_adjustments (warehouse_id, reason, adjusted_by_user_id, created_at) VALUES (%s, %s, %s, %s)", adj_rows)

    adj_item_rows = [
        (i, i, random.randint(-10, 50))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.stock_adjustment_items (adjustment_id, variant_id, quantity_delta) VALUES (%s, %s, %s)", adj_item_rows)

    po_rows = [
        (i, i, f"PO-{now.year}-{i:05d}", random.choice(["approved", "received", "pending"]), round(500.0 + i * 50, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.purchase_orders (supplier_id, warehouse_id, po_number, status, total_cost) VALUES (%s, %s, %s, %s, %s)", po_rows)

    po_item_rows = [
        (i, i, random.randint(10, 500), round(15.0 + i, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.purchase_order_items (purchase_order_id, variant_id, quantity_ordered, unit_cost) VALUES (%s, %s, %s, %s)", po_item_rows)

    transfer_rows = [
        (i, (i % N) + 1, random.choice(["in_transit", "completed", "draft"]), now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.stock_transfers (from_warehouse_id, to_warehouse_id, status, initiated_at) VALUES (%s, %s, %s, %s)", transfer_rows)

    trans_item_rows = [
        (i, i, random.randint(5, 50))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO inventory.stock_transfer_items (transfer_id, variant_id, quantity) VALUES (%s, %s, %s)", trans_item_rows)

    # 5. orders
    cust_rows = [
        (f"CustomerFirst{i}", f"CustomerLast{i}", f"customer{i}@domain.com", f"+1-555-444{i:02d}", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.customers (first_name, last_name, email, phone, registered_at) VALUES (%s, %s, %s, %s, %s)", cust_rows)

    addr_rows = [
        (i, "shipping" if i % 2 == 0 else "billing", f"{i} Main St", f"City {i}", f"9021{i%10}", "USA")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.customer_addresses (customer_id, address_type, street, city, postal_code, country) VALUES (%s, %s, %s, %s, %s, %s)", addr_rows)

    group_rows = [
        (f"Customer Group {i}", round((i % 15) * 1.0, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.customer_groups (name, discount_rate) VALUES (%s, %s)", group_rows)

    grp_member_rows = [
        (i, i)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.customer_group_members (customer_id, group_id) VALUES (%s, %s)", grp_member_rows)

    cart_rows = [
        (i, now, now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.carts (customer_id, created_at, updated_at) VALUES (%s, %s, %s)", cart_rows)

    cart_item_rows = [
        (i, i, random.randint(1, 4), round(25.0 + i, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.cart_items (cart_id, variant_id, quantity, unit_price) VALUES (%s, %s, %s, %s)", cart_item_rows)

    order_rows = [
        (i, f"ORD-{now.year}-{i:05d}", random.choice(["completed", "shipped", "processing", "cancelled"]), round(100.0 + i * 10, 2), round(8.0 + i, 2), round(108.0 + i * 11, 2), now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.orders (customer_id, order_number, order_status, subtotal, tax_total, total_amount, ordered_at) VALUES (%s, %s, %s, %s, %s, %s, %s)", order_rows)

    order_item_rows = [
        (i, i, f"Product {i}", 2, round(50.0 + i, 2), round(100.0 + i * 2, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.order_items (order_id, variant_id, product_name, quantity, unit_price, total_price) VALUES (%s, %s, %s, %s, %s, %s)", order_item_rows)

    order_history_rows = [
        (i, "pending", "completed", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.order_status_history (order_id, old_status, new_status, changed_at) VALUES (%s, %s, %s, %s)", order_history_rows)

    order_discount_rows = [
        (i, f"SAVE{i}", 10.0)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.order_discounts (order_id, code, amount) VALUES (%s, %s, %s)", order_discount_rows)

    shipment_rows = [
        (i, f"TRK-{i:07d}", "FEDEX", "delivered", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.shipments (order_id, tracking_number, carrier_code, status, shipped_at) VALUES (%s, %s, %s, %s, %s)", shipment_rows)

    ship_item_rows = [
        (i, i, 2)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.shipment_items (shipment_id, order_item_id, quantity) VALUES (%s, %s, %s)", ship_item_rows)

    return_rows = [
        (i, f"RET-{i:05d}", "approved", "Customer requested exchange", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.returns (order_id, return_number, status, reason, requested_at) VALUES (%s, %s, %s, %s, %s)", return_rows)

    return_item_rows = [
        (i, i, 1, "unopened")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO orders.return_items (return_id, order_item_id, quantity, condition) VALUES (%s, %s, %s, %s)", return_item_rows)

    # 6. billing
    pay_method_rows = [
        (i, "credit_card", "stripe", f"****-****-****-{i:04d}", i % 2 == 0)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.payment_methods (customer_id, method_type, provider, masked_card, is_default) VALUES (%s, %s, %s, %s, %s)", pay_method_rows)

    invoice_rows = [
        (i, f"INV-{now.year}-{i:05d}", "paid", round(100.0 + i * 5, 2), round(8.0 + i * 0.4, 2), round(108.0 + i * 5.4, 2), now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.invoices (order_id, invoice_number, status, subtotal, tax, total, issued_at) VALUES (%s, %s, %s, %s, %s, %s, %s)", invoice_rows)

    inv_item_rows = [
        (i, f"Item description for invoice {i}", 1, round(108.0 + i * 5.4, 2), round(108.0 + i * 5.4, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.invoice_items (invoice_id, description, quantity, unit_price, amount) VALUES (%s, %s, %s, %s, %s)", inv_item_rows)

    trans_rows = [
        (i, i, f"ch_stripe_{i:08d}", round(108.0 + i * 5.4, 2), "succeeded", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.payment_transactions (invoice_id, payment_method_id, transaction_id, amount, status, processed_at) VALUES (%s, %s, %s, %s, %s, %s)", trans_rows)

    refund_rows = [
        (i, f"re_stripe_{i:08d}", 25.0, "courtesy_credit", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.refunds (transaction_id, refund_reference, amount, reason, refunded_at) VALUES (%s, %s, %s, %s, %s)", refund_rows)

    juris_rows = [
        ("US", f"S{i%50:02d}", f"Jurisdiction {i}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.tax_jurisdictions (country_code, state_code, jurisdiction_name) VALUES (%s, %s, %s)", juris_rows)

    tax_rate_rows = [
        (i, f"Tax Rate {i}", round(5.0 + (i % 8) * 0.5, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.tax_rates (jurisdiction_id, tax_name, rate_percent) VALUES (%s, %s, %s)", tax_rate_rows)

    sub_rows = [
        (i, f"Subscription Plan {i%5}", "active", "monthly", now, now + datetime.timedelta(days=30))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.subscriptions (customer_id, plan_name, status, interval, current_period_start, current_period_end) VALUES (%s, %s, %s, %s, %s, %s)", sub_rows)

    sub_item_rows = [
        (i, f"Seat {i}", 5, 20.00)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.subscription_items (subscription_id, item_name, quantity, unit_amount) VALUES (%s, %s, %s, %s)", sub_item_rows)

    gateway_rows = [
        (f"Gateway {i}", i % 2 == 0, True)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.payment_gateways (name, is_test_mode, is_active) VALUES (%s, %s, %s)", gateway_rows)

    gw_log_rows = [
        (i, "charge.succeeded", json.dumps({"req": f"id_{i}", "code": 200}), now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.gateway_logs (gateway_id, event_type, payload, created_at) VALUES (%s, %s, %s, %s)", gw_log_rows)

    credit_rows = [
        (i, f"CN-{now.year}-{i:05d}", 15.00, now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO billing.credit_notes (invoice_id, credit_number, total_credit, issued_at) VALUES (%s, %s, %s, %s)", credit_rows)

    # 7. crm
    lead_rows = [
        (f"LeadFirst{i}", f"LeadLast{i}", f"lead{i}@company{i}.com", f"Corp {i}", random.choice(["new", "contacted", "qualified", "lost"]), "inbound_web")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.leads (first_name, last_name, email, company, status, source) VALUES (%s, %s, %s, %s, %s, %s)", lead_rows)

    contact_rows = [
        (f"ContactFirst{i}", f"ContactLast{i}", f"contact{i}@client{i}.com", f"+1-555-888{i:02d}", f"VP of Engineering {i}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.contacts (first_name, last_name, email, phone, job_title) VALUES (%s, %s, %s, %s, %s)", contact_rows)

    acc_rows = [
        (f"Enterprise Account {i}", random.choice(["Fintech", "HealthTech", "Retail", "Automotive", "SaaS"]), round(100000.0 * i, 2), f"https://account{i}.com")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.accounts (account_name, industry, annual_revenue, website) VALUES (%s, %s, %s, %s)", acc_rows)

    pipe_rows = [
        (f"Sales Pipeline {i}", True)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.pipelines (name, is_active) VALUES (%s, %s)", pipe_rows)

    stage_rows = [
        (i, f"Stage {i%6}", (i % 6) * 20, i % 6)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.deal_stages (pipeline_id, stage_name, probability_percent, sort_order) VALUES (%s, %s, %s, %s)", stage_rows)

    deal_rows = [
        (i, i, f"Deal Q{((i%4)+1)}-{i}", round(15000.0 + i * 250, 2), now + datetime.timedelta(days=i*2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.deals (account_id, stage_id, deal_name, expected_amount, close_date) VALUES (%s, %s, %s, %s, %s)", deal_rows)

    deal_part_rows = [
        (i, i, "Decision Maker")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.deal_participants (deal_id, contact_id, role_in_deal) VALUES (%s, %s, %s)", deal_part_rows)

    act_rows = [
        (random.choice(["call", "email", "meeting", "demo"]), f"Follow up discussion {i}", now + datetime.timedelta(days=i), i % 2 == 0)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.activities (activity_type, subject, due_date, is_completed) VALUES (%s, %s, %s, %s)", act_rows)

    act_rel_rows = [
        (i, "deal", i)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.activity_relations (activity_id, entity_type, entity_id) VALUES (%s, %s, %s)", act_rel_rows)

    ticket_rows = [
        (f"TICK-{now.year}-{i:05d}", i, f"Issue with checkout flow {i}", random.choice(["high", "medium", "low"]), random.choice(["open", "resolved", "pending"]), now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.support_tickets (ticket_code, customer_id, subject, priority, status, created_at) VALUES (%s, %s, %s, %s, %s, %s)", ticket_rows)

    tmsg_rows = [
        (i, "agent", i, f"Hello, we have looked into ticket {i} and resolved it.", now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.ticket_messages (ticket_id, sender_type, sender_id, body, sent_at) VALUES (%s, %s, %s, %s, %s)", tmsg_rows)

    ttag_rows = [
        (i, f"tag_urgent_{i%10}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO crm.ticket_tags (ticket_id, tag_name) VALUES (%s, %s)", ttag_rows)

    # 8. marketing
    camp_rows = [
        (f"Campaign {i}", random.choice(["email", "social", "ppc", "event"]), round(1000.0 * i, 2), now, now + datetime.timedelta(days=30))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.campaigns (name, campaign_type, budget, start_date, end_date) VALUES (%s, %s, %s, %s, %s)", camp_rows)

    channel_rows = [
        (i, random.choice(["Google Ads", "Meta", "LinkedIn", "Twitter/X", "Newsletter"]), round(250.0 * i, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.campaign_channels (campaign_id, channel_name, spend) VALUES (%s, %s, %s)", channel_rows)

    tmpl_rows = [
        (f"Template {i}", f"Exciting news about product update {i}!", f"<html><body><h1>Hi there #{i}</h1></body></html>")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.email_templates (template_name, subject_line, body_html) VALUES (%s, %s, %s)", tmpl_rows)

    seg_rows = [
        (f"Segment VIP {i}", json.dumps({"min_spend": 500, "tier": i % 3}), 1500)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.segments (segment_name, filter_criteria, members_count) VALUES (%s, %s, %s)", seg_rows)

    nl_rows = [
        (i, i, f"Newsletter Edition #{i}", now + datetime.timedelta(days=i))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.newsletters (campaign_id, template_id, title, scheduled_at) VALUES (%s, %s, %s, %s)", nl_rows)

    sub_rows = [
        (f"subscriber{i}@news.com", True, now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.subscribers (email, is_subscribed, opted_in_at) VALUES (%s, %s, %s)", sub_rows)

    seg_mem_rows = [
        (i, i)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.segment_members (segment_id, subscriber_id) VALUES (%s, %s)", seg_mem_rows)

    coupon_rows = [
        (f"COUPON{i:03d}", "percentage", 15.0, 500, now + datetime.timedelta(days=90))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.coupons (code, discount_type, value, max_uses, valid_until) VALUES (%s, %s, %s, %s, %s)", coupon_rows)

    redemp_rows = [
        (i, i, now)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.coupon_redemptions (coupon_id, customer_id, redeemed_at) VALUES (%s, %s, %s)", redemp_rows)

    affil_rows = [
        (f"Affiliate Partner {i}", f"REF{i:04d}", 10.0)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO marketing.affiliates (partner_name, referral_code, commission_rate) VALUES (%s, %s, %s)", affil_rows)

    # 9. logistics
    carrier_rows = [
        (f"Carrier {i}", f"https://api.carrier{i}.com/v1", f"https://track.carrier{i}.com?id={{tracking_number}}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.carriers (carrier_name, api_endpoint, tracking_url_template) VALUES (%s, %s, %s)", carrier_rows)

    serv_rows = [
        (i, f"SRV_{i:02d}", f"Priority Express {i}", 2)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.service_levels (carrier_id, service_code, service_name, max_transit_days) VALUES (%s, %s, %s, %s)", serv_rows)

    zone_rows = [
        (f"Zone {i}", round(5.0 + i * 0.5, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.shipping_zones (zone_name, base_cost) VALUES (%s, %s)", zone_rows)

    zpost_rows = [
        (i, f"902{i%10}")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.zone_postal_codes (zone_id, postal_prefix) VALUES (%s, %s)", zpost_rows)

    rate_rows = [
        (i, i, round(2.5 + i * 0.1, 2), round(12.0 + i * 0.5, 2))
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.shipping_rate_tables (service_id, zone_id, weight_kg, rate) VALUES (%s, %s, %s, %s)", rate_rows)

    veh_rows = [
        (f"51A-{i:05d}", random.choice(["Van", "Truck 5T", "Semi-Trailer", "Cargo Scooter"]), 25.5, 3500.0)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.vehicles (license_plate, vehicle_type, max_volume_m3, max_weight_kg) VALUES (%s, %s, %s, %s)", veh_rows)

    driver_rows = [
        (f"Driver Name {i}", f"DL-2026-{i:05d}", f"+1-555-777{i:02d}", True)
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.drivers (driver_name, license_number, phone, is_available) VALUES (%s, %s, %s, %s)", driver_rows)

    route_rows = [
        (i, i, now.date(), "completed")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.delivery_routes (vehicle_id, driver_id, route_date, status) VALUES (%s, %s, %s, %s)", route_rows)

    stop_rows = [
        (i, i % 10 + 1, f"{i} Delivery Road, Zone {i}", f"Recipient {i}", "delivered")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.delivery_stops (route_id, stop_sequence, address, recipient_name, delivery_status) VALUES (%s, %s, %s, %s, %s)", stop_rows)

    track_rows = [
        (f"TRK-{i:07d}", now, f"Hub Center {i}", f"Package arrived at sort facility {i}", "ARRIVED")
        for i in range(1, N + 1)
    ]
    cursor.executemany("INSERT INTO logistics.tracking_events (tracking_number, event_time, location, description, status_code) VALUES (%s, %s, %s, %s, %s)", track_rows)

    print("Data seeding completed successfully!")

def verify(cursor):
    cursor.execute("""
        SELECT table_schema, table_name
        FROM information_schema.tables
        WHERE table_schema IN ('design_system', 'identity', 'catalog', 'inventory', 'orders', 'billing', 'crm', 'marketing', 'logistics')
        ORDER BY table_schema, table_name;
    """)
    tables = cursor.fetchall()
    print(f"Total tables created across 9 schemas: {len(tables)}")
    assert len(tables) == 115, f"Expected 115 tables, found {len(tables)}"

    min_count = 999999
    max_count = 0
    total_records = 0
    for schema, table in tables:
        cursor.execute(f'SELECT COUNT(*) FROM "{schema}"."{table}";')
        cnt = cursor.fetchone()[0]
        min_count = min(min_count, cnt)
        max_count = max(max_count, cnt)
        total_records += cnt

    print(f"Verified {len(tables)} tables: exactly {min_count} records per table (Total: {total_records} records).")
    return len(tables), total_records

def main():
    conn = get_connection()
    conn.autocommit = False
    cursor = conn.cursor()
    try:
        create_schema(cursor)
        conn.commit()
        seed_data(cursor)
        conn.commit()
        num_tables, total_records = verify(cursor)
        print(f"SUCCESS! Database enterprise_db initialized with {num_tables} tables and {total_records} records.")
    except Exception as e:
        conn.rollback()
        print(f"Error: {e}")
        sys.exit(1)
    finally:
        cursor.close()
        conn.close()

if __name__ == "__main__":
    main()
