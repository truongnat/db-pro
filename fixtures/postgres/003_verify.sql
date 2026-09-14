-- Verify deterministic representative PostgreSQL fixture row counts.
-- Run after 001_schema.sql and 002_seed.sql with ON_ERROR_STOP enabled.

DO $$
BEGIN
    IF (SELECT COUNT(*) FROM categories) <> 4 THEN
        RAISE EXCEPTION 'fixture verification failed: categories count != 4';
    END IF;

    IF (SELECT COUNT(*) FROM products) <> 4 THEN
        RAISE EXCEPTION 'fixture verification failed: products count != 4';
    END IF;

    IF (SELECT COUNT(*) FROM orders) <> 4 THEN
        RAISE EXCEPTION 'fixture verification failed: orders count != 4';
    END IF;

    IF (SELECT COUNT(*) FROM order_items) <> 6 THEN
        RAISE EXCEPTION 'fixture verification failed: order_items count != 6';
    END IF;
END
$$;

-- Decoder matrix rows are part of the deterministic fixture (#59).
DO $$
BEGIN
    IF (SELECT COUNT(*) FROM decoder_matrix) <> 2 THEN
        RAISE EXCEPTION 'fixture verification failed: decoder_matrix count != 2';
    END IF;

    IF (SELECT COUNT(*) FROM decoder_matrix WHERE id = 1 AND flag AND quantity = 7) <> 1 THEN
        RAISE EXCEPTION 'fixture verification failed: decoder_matrix populated row missing';
    END IF;
END
$$;
