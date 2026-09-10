-- PostgreSQL smoke fixture teardown.
-- Drops all smoke objects in dependency order.
-- Run to reset the database for a clean re-seed.

DROP VIEW IF EXISTS smoke_order_summary;
DROP VIEW IF EXISTS smoke_active_users;

DROP TRIGGER IF EXISTS smoke_users_updated_at ON smoke_users;
DROP FUNCTION IF EXISTS smoke_update_timestamp();

DROP TABLE IF EXISTS smoke_order_items;
DROP TABLE IF EXISTS smoke_orders;
DROP TABLE IF EXISTS smoke_users;
DROP TABLE IF EXISTS smoke_audit_log;
DROP TABLE IF EXISTS smoke_empty;
DROP TABLE IF EXISTS smoke_documents;
DROP TABLE IF EXISTS smoke_employees;
DROP TABLE IF EXISTS smoke_products;
DROP TABLE IF EXISTS smoke_network;
DROP TABLE IF EXISTS "smoke_Ünïcödé";

DROP SEQUENCE IF EXISTS smoke_event_seq;
DROP TYPE IF EXISTS smoke_status;
