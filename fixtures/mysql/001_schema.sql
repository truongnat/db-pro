-- MySQL 8 fixture schema for DB Pro live provider tests (#235).
-- Covers: one column per class the provider-value contract distinguishes, a
-- composite primary key, a foreign key, an index, a view, a trigger and a routine.

-- Pin the session charset: the `mysql` client inside the server image does not default to
-- utf8mb4, and without this the file's UTF-8 bytes are read as latin1 and the seeded text
-- is stored double-encoded (measured: `HEX(label)` came back as the UTF-8 of the mojibake).
SET NAMES utf8mb4;

-- ── Categories ─────────────────────────────────────────────────
CREATE TABLE categories (
    id INT PRIMARY KEY,
    name VARCHAR(100) NOT NULL
);

-- ── Order items: composite PK + FK + index ─────────────────────
CREATE TABLE order_items (
    order_id INT NOT NULL,
    line_no INT NOT NULL,
    category_id INT,
    sku VARCHAR(50),
    quantity INT UNSIGNED,
    PRIMARY KEY (order_id, line_no),
    CONSTRAINT fk_order_items_category FOREIGN KEY (category_id) REFERENCES categories (id),
    INDEX idx_order_items_sku (sku)
);

-- ── Audit table written by the trigger below ───────────────────
CREATE TABLE order_item_audit (
    audit_id BIGINT AUTO_INCREMENT PRIMARY KEY,
    order_id INT NOT NULL,
    line_no INT NOT NULL,
    action VARCHAR(16) NOT NULL
);

-- ── Decoder matrix ─────────────────────────────────────────────
-- Row 1 carries a value for every class; row 2 is NULL in every nullable
-- family, so a single SELECT measures the whole decoder surface.
CREATE TABLE decoder_matrix (
    id SMALLINT PRIMARY KEY,
    flag BOOLEAN,                          -- TINYINT(1)
    tiny_count TINYINT,                    -- signed minimum
    small_count SMALLINT,                  -- signed minimum
    medium_count MEDIUMINT,
    count_value INT,
    big_count BIGINT,                      -- signed maximum (i64::MAX)
    tiny_unsigned TINYINT UNSIGNED,
    int_unsigned INT UNSIGNED,
    big_unsigned BIGINT UNSIGNED,          -- above i64::MAX
    ratio FLOAT,
    precise_ratio DOUBLE,
    amount DECIMAL(30, 10),                -- more precision than f64 can hold
    scaled_amount DECIMAL(10, 2),          -- declared scale must survive
    calendar_date DATE,
    wall_time TIME(6),                     -- inside one day
    long_time TIME(6),                     -- MySQL TIME beyond 24 h
    negative_time TIME(6),                 -- MySQL TIME is signed
    local_stamp DATETIME(6),               -- wall clock, no zone
    instant TIMESTAMP(6),                  -- instant, converted by the session zone
    year_value YEAR,
    bit_one BIT(1),
    bit_eight BIT(8),
    bit_sixtyfour BIT(64),                 -- wider than the bool probe can hold
    token CHAR(36),
    label VARCHAR(64),
    binary_collation VARCHAR(32) COLLATE utf8mb4_bin,
    body TEXT,
    tiny_body TINYTEXT,
    medium_body MEDIUMTEXT,
    long_body LONGTEXT,
    raw_bytes BINARY(8),
    raw_varbinary VARBINARY(16),
    blob_data BLOB,                        -- bytes that are not valid UTF-8
    tiny_raw TINYBLOB,                     -- bytes that happen to be valid UTF-8
    long_raw LONGBLOB,
    status ENUM('pending', 'processing', 'shipped', 'delivered', 'cancelled'),
    tags SET('alpha', 'beta', 'gamma'),
    doc JSON,
    geo GEOMETRY,
    missing VARCHAR(32)                    -- always NULL
);

-- ── View ───────────────────────────────────────────────────────
CREATE VIEW v_order_item_totals AS
SELECT oi.order_id, c.name AS category_name, SUM(oi.quantity) AS total_quantity
FROM order_items oi
LEFT JOIN categories c ON c.id = oi.category_id
GROUP BY oi.order_id, c.name;

-- ── Trigger ────────────────────────────────────────────────────
DELIMITER //
CREATE TRIGGER order_items_after_insert
AFTER INSERT ON order_items
FOR EACH ROW
BEGIN
    INSERT INTO order_item_audit (order_id, line_no, action)
    VALUES (NEW.order_id, NEW.line_no, 'insert');
END//
DELIMITER ;

-- ── Routine ────────────────────────────────────────────────────
DELIMITER //
CREATE FUNCTION order_item_total(p_order_id INT)
RETURNS INT
DETERMINISTIC
READS SQL DATA
BEGIN
    DECLARE total INT;
    SELECT COALESCE(SUM(quantity), 0) INTO total FROM order_items WHERE order_id = p_order_id;
    RETURN total;
END//
DELIMITER ;
