use serde::{Deserialize, Serialize};

mod string_i64 {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<i64>().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum QueryParam {
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "bool")]
    Bool(bool),
    #[serde(rename = "int64")]
    Int64(#[serde(with = "string_i64")] i64),
    #[serde(rename = "float64")]
    Float64(f64),
    #[serde(rename = "decimal")]
    Decimal(String),
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "bytes")]
    Bytes(Vec<u8>),
    #[serde(rename = "uuid")]
    Uuid(String),
    #[serde(rename = "datetime")]
    DateTime(String),
    #[serde(rename = "time")]
    Time(String),
    #[serde(rename = "interval")]
    Interval(String),
    #[serde(rename = "inet")]
    Inet(String),
    #[serde(rename = "json")]
    Json(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CellValue {
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "bool")]
    Bool(bool),
    #[serde(rename = "int64")]
    Int64(#[serde(with = "string_i64")] i64),
    #[serde(rename = "float64")]
    Float64(f64),
    #[serde(rename = "decimal")]
    Decimal(String),
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "bytes")]
    Bytes(Vec<u8>),
    #[serde(rename = "uuid")]
    Uuid(String),
    #[serde(rename = "datetime")]
    DateTime(String),
    #[serde(rename = "date")]
    Date(String),
    #[serde(rename = "time")]
    Time(String),
    #[serde(rename = "interval")]
    Interval(String),
    #[serde(rename = "inet")]
    Inet(String),
    #[serde(rename = "json")]
    Json(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMeta {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row(pub Vec<CellValue>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Row>,
    pub row_count: u64,
    pub duration_ms: u64,
}

impl QueryResult {
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
            duration_ms: 0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let col_count = self.columns.len();
        if self.columns.is_empty() && !self.rows.is_empty() {
            return Err("result has returned rows but no columns".into());
        }
        for (i, row) in self.rows.iter().enumerate() {
            if row.0.len() != col_count {
                return Err(format!("row {i} has {} cells but expected {col_count}", row.0.len()));
            }
        }
        if !self.columns.is_empty() && self.row_count != self.rows.len() as u64 {
            return Err(format!(
                "row_count {} does not match {} returned rows",
                self.row_count,
                self.rows.len()
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum QueryError {
    #[error("connection not found: {connection_id}")]
    ConnectionNotFound { connection_id: String },

    #[error("syntax error at line {line}: {message}")]
    SyntaxError { line: u32, message: String },

    #[error("permission denied on table: {table}")]
    PermissionDenied { table: String },

    #[error("query timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("connection lost")]
    ConnectionLost,

    #[error("multi-statement execution is disabled")]
    MultiStatementDisabled,

    #[error("unsupported parameter type: {0}")]
    UnsupportedParameterType(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl QueryError {
    pub fn code(&self) -> &'static str {
        match self {
            QueryError::ConnectionNotFound { .. } => "CONNECTION_NOT_FOUND",
            QueryError::SyntaxError { .. } => "SYNTAX_ERROR",
            QueryError::PermissionDenied { .. } => "PERMISSION_DENIED",
            QueryError::Timeout { .. } => "QUERY_TIMEOUT",
            QueryError::ConnectionLost => "CONNECTION_LOST",
            QueryError::MultiStatementDisabled => "MULTI_STATEMENT_DISABLED",
            QueryError::UnsupportedParameterType(_) => "UNSUPPORTED_PARAMETER_TYPE",
            QueryError::Validation(_) => "VALIDATION_ERROR",
            QueryError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn message_id(&self) -> &'static str {
        match self {
            QueryError::ConnectionNotFound { .. } => "error.connection.not_found",
            QueryError::SyntaxError { .. } => "error.query.syntax",
            QueryError::PermissionDenied { .. } => "error.query.permission",
            QueryError::Timeout { .. } => "error.query.timeout",
            QueryError::ConnectionLost => "error.connection.lost",
            QueryError::MultiStatementDisabled => "error.query.multi_statement",
            QueryError::UnsupportedParameterType(_) => "error.query.unsupported_param",
            QueryError::Validation(_) => "error.validation",
            QueryError::Internal(_) => "error.internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_result_validate_empty() {
        let result = QueryResult::empty();
        assert!(result.validate().is_ok());
    }

    #[test]
    fn query_result_validate_matching_columns() {
        let result = QueryResult {
            columns: vec![
                ColumnMeta {
                    name: "id".into(),
                    data_type: "int".into(),
                    nullable: false,
                },
                ColumnMeta {
                    name: "name".into(),
                    data_type: "text".into(),
                    nullable: true,
                },
            ],
            rows: vec![Row(vec![CellValue::Int64(1), CellValue::Text("alice".into())])],
            row_count: 1,
            duration_ms: 0,
        };
        assert!(result.validate().is_ok());
    }

    #[test]
    fn query_result_validate_cell_count_mismatch() {
        let result = QueryResult {
            columns: vec![
                ColumnMeta {
                    name: "id".into(),
                    data_type: "int".into(),
                    nullable: false,
                },
                ColumnMeta {
                    name: "name".into(),
                    data_type: "text".into(),
                    nullable: true,
                },
            ],
            rows: vec![Row(vec![CellValue::Int64(1)])],
            row_count: 1,
            duration_ms: 0,
        };
        assert!(result.validate().is_err());
    }

    #[test]
    fn query_result_validate_row_count_mismatch() {
        let result = QueryResult {
            columns: vec![ColumnMeta {
                name: "id".into(),
                data_type: "int".into(),
                nullable: false,
            }],
            rows: vec![Row(vec![CellValue::Int64(1)])],
            row_count: 2,
            duration_ms: 0,
        };

        assert!(matches!(result.validate(), Err(message) if message.contains("row_count")));
    }

    #[test]
    fn query_result_validate_affected_rows_without_columns() {
        let result = QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 3,
            duration_ms: 0,
        };

        assert!(result.validate().is_ok());
    }

    #[test]
    fn query_result_validate_rejects_rows_without_columns() {
        let result = QueryResult {
            columns: Vec::new(),
            rows: vec![Row(Vec::new())],
            row_count: 1,
            duration_ms: 0,
        };

        assert!(matches!(result.validate(), Err(message) if message.contains("no columns")));
    }

    #[test]
    fn int64_serializes_as_string_for_lossless_ipc() {
        let cell = CellValue::Int64(9_007_199_254_740_993); // 2^53 + 1
        let json = serde_json::to_string(&cell).unwrap();
        assert!(
            json.contains("\"9007199254740993\""),
            "i64 must serialize as string: {json}"
        );
        let back: CellValue = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, CellValue::Int64(9_007_199_254_740_993)));
    }

    #[test]
    fn int64_boundary_values_round_trip() {
        for &v in &[
            0i64,
            1,
            -1,
            i64::MAX,
            i64::MIN,
            (1i64 << 53) - 1,
            1i64 << 53,
            (1i64 << 53) + 1,
        ] {
            let cell = CellValue::Int64(v);
            let json = serde_json::to_string(&cell).unwrap();
            let back: CellValue = serde_json::from_str(&json).unwrap();
            assert!(
                matches!(back, CellValue::Int64(x) if x == v),
                "round-trip failed for {v}"
            );
        }
    }

    #[test]
    fn query_param_int64_string_round_trip() {
        let param = QueryParam::Int64(i64::MAX);
        let json = serde_json::to_string(&param).unwrap();
        let back: QueryParam = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, QueryParam::Int64(x) if x == i64::MAX));
    }

    #[test]
    fn typed_query_params_round_trip_through_ipc_json() {
        let params = vec![
            QueryParam::Time("12:34:56.123456".into()),
            QueryParam::Interval("1 days 02:00:00".into()),
            QueryParam::Inet("192.0.2.1/24".into()),
        ];
        let json = serde_json::to_string(&params).unwrap();
        let back: Vec<QueryParam> = serde_json::from_str(&json).unwrap();

        assert!(matches!(
            back.as_slice(),
            [
                QueryParam::Time(time),
                QueryParam::Interval(interval),
                QueryParam::Inet(address)
            ] if time == "12:34:56.123456"
                && interval == "1 days 02:00:00"
                && address == "192.0.2.1/24"
        ));
    }
}
