use std::fs;
use std::path::Path;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Executor;

use crate::models::{ConnectionProfile, DbEngine};

pub const SAMPLE_CONNECTION_ID: &str = "sample-sqlite";
pub const SAMPLE_CONNECTION_NAME: &str = "Sample SQLite";

pub fn sample_connection_profile(path: &Path) -> ConnectionProfile {
    ConnectionProfile {
        id: SAMPLE_CONNECTION_ID.to_string(),
        name: SAMPLE_CONNECTION_NAME.to_string(),
        engine: DbEngine::Sqlite,
        path: Some(path.to_string_lossy().to_string()),
        host: None,
        port: None,
        database: None,
        username: None,
        password: None,
    }
}

pub async fn ensure_sample_sqlite(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "Failed to create sample SQLite directory '{}': {err}",
                parent.display()
            )
        })?;
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|err| format!("Failed to open sample SQLite database: {err}"))?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS customers (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            tier TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .await
    .map_err(|err| format!("Failed to create customers table: {err}"))?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            price_cents INTEGER NOT NULL,
            in_stock INTEGER NOT NULL DEFAULT 1
        )",
    )
    .await
    .map_err(|err| format!("Failed to create products table: {err}"))?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY,
            customer_id INTEGER NOT NULL,
            ordered_at TEXT NOT NULL,
            status TEXT NOT NULL,
            total_cents INTEGER NOT NULL,
            FOREIGN KEY(customer_id) REFERENCES customers(id)
        )",
    )
    .await
    .map_err(|err| format!("Failed to create orders table: {err}"))?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS order_items (
            order_id INTEGER NOT NULL,
            product_id INTEGER NOT NULL,
            quantity INTEGER NOT NULL,
            unit_price_cents INTEGER NOT NULL,
            PRIMARY KEY(order_id, product_id),
            FOREIGN KEY(order_id) REFERENCES orders(id),
            FOREIGN KEY(product_id) REFERENCES products(id)
        )",
    )
    .await
    .map_err(|err| format!("Failed to create order_items table: {err}"))?;

    pool.execute(
        "CREATE VIEW IF NOT EXISTS order_summary AS
         SELECT
            o.id AS order_id,
            c.name AS customer_name,
            o.status,
            o.ordered_at,
            ROUND(o.total_cents / 100.0, 2) AS total_usd
         FROM orders o
         JOIN customers c ON c.id = o.customer_id",
    )
    .await
    .map_err(|err| format!("Failed to create order_summary view: {err}"))?;

    pool.execute(
        "INSERT OR IGNORE INTO customers (id, name, email, tier, created_at) VALUES
            (1, 'Alice Nguyen', 'alice@example.com', 'gold', '2026-01-02 08:30:00'),
            (2, 'Bao Tran', 'bao@example.com', 'standard', '2026-01-11 10:15:00'),
            (3, 'Chi Le', 'chi@example.com', 'platinum', '2026-02-05 13:45:00')",
    )
    .await
    .map_err(|err| format!("Failed to seed customers: {err}"))?;

    pool.execute(
        "INSERT OR IGNORE INTO products (id, name, category, price_cents, in_stock) VALUES
            (1, 'Mechanical Keyboard', 'Accessories', 12900, 1),
            (2, 'USB-C Hub', 'Accessories', 5900, 1),
            (3, '4K Monitor', 'Display', 32900, 1),
            (4, 'Ergonomic Mouse', 'Accessories', 7900, 0)",
    )
    .await
    .map_err(|err| format!("Failed to seed products: {err}"))?;

    pool.execute(
        "INSERT OR IGNORE INTO orders (id, customer_id, ordered_at, status, total_cents) VALUES
            (1001, 1, '2026-02-14 09:12:00', 'completed', 18800),
            (1002, 2, '2026-02-18 16:40:00', 'processing', 32900),
            (1003, 3, '2026-02-26 11:21:00', 'completed', 45800)",
    )
    .await
    .map_err(|err| format!("Failed to seed orders: {err}"))?;

    pool.execute(
        "INSERT OR IGNORE INTO order_items (order_id, product_id, quantity, unit_price_cents) VALUES
            (1001, 1, 1, 12900),
            (1001, 2, 1, 5900),
            (1002, 3, 1, 32900),
            (1003, 1, 1, 12900),
            (1003, 2, 1, 5900),
            (1003, 4, 1, 7900)",
    )
    .await
    .map_err(|err| format!("Failed to seed order_items: {err}"))?;

    pool.close().await;
    Ok(())
}
