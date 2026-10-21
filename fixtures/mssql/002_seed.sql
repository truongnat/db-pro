-- Deterministic seed for the SQL Server fixture. Row 1 carries a value for every
-- class the shared decoder contract measures; row 2 is NULL in every nullable
-- family. Values match the PostgreSQL/MySQL fixtures wherever the classes overlap
-- so one set of assertions serves all three providers.
-- Idempotent: the matrix is rebuilt from empty on every load.

DELETE FROM dbo.decoder_matrix;

INSERT INTO dbo.decoder_matrix (
    id, flag, count_value, big_count, ratio, precise_ratio, amount,
    calendar_date, wall_time, local_stamp, instant, token, doc, blob_data, status, missing
) VALUES (
    1, 1, 2147483647, 9223372036854775807, 1.5, 0.1, 12345678901234567890.1234,
    '2024-03-15', '10:20:30.123456',
    '2024-03-15 10:20:30.123456', '2024-03-15 10:20:30.123456 +00:00',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    N'{"a": 1, "b": [true, null]}', 0xDEADBEEF2024, N'shipped', NULL
), (
    2, 0, NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL
);
