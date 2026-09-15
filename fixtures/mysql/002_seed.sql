-- Deterministic seed for the MySQL fixture. Every value is chosen to expose a
-- specific decoder decision; the decoder matrix is asserted by the live tests.
-- The charset is pinned so the unicode row is stored as written.

SET NAMES utf8mb4;

INSERT INTO categories (id, name) VALUES (1, 'Hardware'), (2, 'Software');

INSERT INTO order_items (order_id, line_no, category_id, sku, quantity) VALUES
    (1001, 1, 1, 'KB-001', 2),
    (1001, 2, 2, 'SW-004', 5),
    (1002, 1, NULL, 'KB-002', NULL);

-- Row 1: a value for every class. Row 2: NULL in every nullable family.
INSERT INTO decoder_matrix (
    id, flag, tiny_count, small_count, medium_count, count_value, big_count,
    tiny_unsigned, int_unsigned, big_unsigned, ratio, precise_ratio,
    amount, scaled_amount, calendar_date, wall_time, long_time, negative_time,
    local_stamp, instant, year_value, bit_one, bit_eight, bit_sixtyfour,
    token, label, binary_collation, body, tiny_body, medium_body, long_body,
    raw_bytes, raw_varbinary, blob_data, tiny_raw, long_raw,
    status, tags, doc, geo, missing
) VALUES (
    1, TRUE, -128, -32768, 8388607, 2147483647, 9223372036854775807,
    255, 4294967295, 18446744073709551615, 1.5, 0.1,
    12345678901234567890.1234567890, 10.50, '2024-03-15', '10:20:30.123456',
    '800:59:59.123456', '-100:30:15.500000',
    '2024-03-15 10:20:30.123456', '2024-03-15 10:20:30.123456', 2024,
    b'1', b'10101010',
    b'1111111111111111111111111111111111111111111111111111111111111111',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'Ünïcödé ✓ 日本語',
    'bin-collated',
    'plain text body', 'tiny', 'medium', REPEAT('long text ', 40),
    X'DEADBEEF00112233', X'FF00FE01', X'FF00FE01',
    'hello', REPEAT(X'AB', 64),
    'shipped', 'alpha,gamma', '{"a": 1, "b": [true, null]}',
    ST_GeomFromText('POINT(1 2)'), NULL
), (
    2, NULL, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL
);
