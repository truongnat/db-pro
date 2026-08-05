use db_pro_core::domain::connection::{ConnectionConfig, SslMode};
use db_pro_core::domain::error::DbError;
use sqlx::postgres::PgConnectOptions;

pub fn build_options(config: &ConnectionConfig, password: &str) -> Result<PgConnectOptions, DbError> {
    let ssl_mode = match config.ssl_mode {
        SslMode::Disable => sqlx::postgres::PgSslMode::Disable,
        SslMode::Require => sqlx::postgres::PgSslMode::Require,
        SslMode::VerifyCa => sqlx::postgres::PgSslMode::VerifyCa,
        SslMode::VerifyFull => sqlx::postgres::PgSslMode::VerifyFull,
    };

    Ok(PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(password)
        .database(&config.database)
        .ssl_mode(ssl_mode))
}
