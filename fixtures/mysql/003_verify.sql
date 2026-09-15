-- Verify the deterministic MySQL fixture after 001_schema.sql and 002_seed.sql.
-- Raises SQLSTATE 45000, which stops the mysql client, when a count is wrong.

SET NAMES utf8mb4;

DELIMITER //
CREATE PROCEDURE assert_fixture_counts()
BEGIN
    DECLARE total INT;

    SELECT COUNT(*) INTO total FROM decoder_matrix;
    IF total <> 2 THEN
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'fixture verification failed: decoder_matrix count != 2';
    END IF;

    SELECT COUNT(*) INTO total FROM categories;
    IF total <> 2 THEN
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'fixture verification failed: categories count != 2';
    END IF;

    SELECT COUNT(*) INTO total FROM order_items;
    IF total <> 3 THEN
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'fixture verification failed: order_items count != 3';
    END IF;

    SELECT COUNT(*) INTO total FROM order_item_audit;
    IF total <> 3 THEN
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'fixture verification failed: order_item_audit count != 3 (trigger did not fire)';
    END IF;

    SELECT COUNT(*) INTO total FROM decoder_matrix WHERE missing IS NULL;
    IF total <> 2 THEN
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'fixture verification failed: decoder_matrix NULL row != 2';
    END IF;

    -- The unicode row must be stored as written: a client charset that is not utf8mb4
    -- reads this file's bytes as latin1 and stores the text double-encoded.
    IF (SELECT HEX(label) FROM decoder_matrix WHERE id = 1) <> 'C39C6EC3AF63C3B664C3A920E29C9320E697A5E69CACE8AA9E' THEN
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'fixture verification failed: label is not stored as utf8mb4';
    END IF;
END//
DELIMITER ;

CALL assert_fixture_counts();
DROP PROCEDURE assert_fixture_counts;
