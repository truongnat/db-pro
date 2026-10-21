-- SQL Server 2022 fixture schema for DB Pro live provider conformance (#260).
-- Mirrors the `decoder_matrix` of fixtures/postgres and fixtures/mysql for the
-- types the lossless TDS mapper (`crates/infrastructure/src/sqlserver/query_mapper.rs`)
-- decodes: int/bigint -> Int64, real/float -> Float64, decimal -> Decimal (string),
-- date -> Date, time -> Time, datetime2 -> Timestamp, datetimeoffset -> TimestampTz,
-- uniqueidentifier -> Uuid, nvarchar -> Text, varbinary -> Bytes.
-- SQL Server has no native enum/json decoder arm: `status` is an NVARCHAR with a
-- CHECK list, `doc` is JSON text with an ISJSON CHECK — the conformance suite
-- expects Text for both (driver-gated) instead of weakening the Json assertion.

IF OBJECT_ID('dbo.decoder_matrix', 'U') IS NOT NULL DROP TABLE dbo.decoder_matrix;

CREATE TABLE dbo.decoder_matrix (
    id            INT PRIMARY KEY,                 -- int  -> Int64, never Bool
    flag          BIT NOT NULL,                    -- bit  -> Bool
    count_value   INT,                             -- int  -> Int64
    big_count     BIGINT,                          -- bigint -> Int64 (i64::MAX exact)
    ratio         REAL,                            -- float4 -> Float64
    precise_ratio FLOAT,                           -- float8 -> Float64
    amount        DECIMAL(24, 4),                  -- exact numeric -> Decimal string, scale kept
    calendar_date DATE,                            -- date -> Date
    wall_time     TIME(6),                         -- time -> Time
    local_stamp   DATETIME2(6),                    -- datetime2 -> Timestamp, no invented offset
    instant       DATETIMEOFFSET(6),               -- datetimeoffset -> TimestampTz with offset
    token         UNIQUEIDENTIFIER,                -- uniqueidentifier -> Uuid
    doc           NVARCHAR(MAX),                   -- JSON text -> Text (ISJSON CHECK)
    blob_data     VARBINARY(MAX),                  -- varbinary -> Bytes, byte-exact
    status        NVARCHAR(32),                    -- enum stand-in -> Text
    missing       NVARCHAR(MAX),                   -- NULL carrier
    CONSTRAINT chk_decoder_matrix_doc_is_json CHECK (doc IS NULL OR ISJSON(doc) = 1),
    CONSTRAINT chk_decoder_matrix_status CHECK (status IS NULL OR status IN (N'pending', N'shipped', N'cancelled'))
);
