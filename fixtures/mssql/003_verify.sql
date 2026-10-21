-- Verify the deterministic SQL Server fixture after 001_schema.sql and 002_seed.sql.
-- THROW aborts the batch; run sqlcmd with -b so the non-zero exit stops the loader.

IF (SELECT COUNT(*) FROM dbo.decoder_matrix) <> 2
    THROW 50001, N'fixture verification failed: decoder_matrix count != 2', 1;

IF (SELECT COUNT(*) FROM dbo.decoder_matrix WHERE big_count = 9223372036854775807) <> 1
    THROW 50002, N'fixture verification failed: big_count seed not exact', 1;

IF (SELECT COUNT(*) FROM dbo.decoder_matrix WHERE blob_data = 0xDEADBEEF2024) <> 1
    THROW 50003, N'fixture verification failed: blob_data seed missing', 1;
