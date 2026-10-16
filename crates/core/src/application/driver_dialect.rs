use crate::domain::connection::DriverType;
use crate::ports::SqlDialect;

/// Quoting and placeholders for the connection the user is actually on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverDialect {
    driver: DriverType,
}

impl DriverDialect {
    pub(crate) fn new(driver: DriverType) -> Self {
        Self { driver }
    }

    pub fn from_name(driver: &str) -> Self {
        Self::new(driver_type_from_name(driver))
    }
}

pub fn driver_type_from_name(driver: &str) -> DriverType {
    let name = driver.to_ascii_lowercase();
    if name.contains("mysql") || name.contains("maria") {
        DriverType::Mysql
    } else if name.contains("sqlserver") || name.contains("mssql") || name.contains("sql server") {
        DriverType::SqlServer
    } else if name.contains("sqlite") {
        DriverType::SQLite
    } else {
        DriverType::Postgres
    }
}

impl SqlDialect for DriverDialect {
    fn placeholder(&self, index: usize) -> String {
        match self.driver {
            DriverType::Postgres => format!("${index}"),
            DriverType::SqlServer => format!("@p{index}"),
            DriverType::Mysql | DriverType::SQLite => "?".to_owned(),
        }
    }

    fn quote_identifier(&self, name: &str) -> String {
        match self.driver {
            DriverType::Mysql => format!("`{}`", name.replace('`', "``")),
            DriverType::SqlServer => format!("[{}]", name.replace(']', "]]")),
            DriverType::Postgres | DriverType::SQLite => format!("\"{}\"", name.replace('"', "\"\"")),
        }
    }

    fn text_like(&self, column: &str, placeholder: &str) -> String {
        match self.driver {
            DriverType::Mysql => format!("CAST({column} AS CHAR) LIKE {placeholder}"),
            DriverType::SqlServer => format!("CAST({column} AS NVARCHAR(MAX)) LIKE {placeholder}"),
            DriverType::Postgres | DriverType::SQLite => format!("CAST({column} AS TEXT) LIKE {placeholder}"),
        }
    }
}
