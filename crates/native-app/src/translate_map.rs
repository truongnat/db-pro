//! Shared UI↔domain value mapping helpers.
//! UiCommand → RuntimeCommand mapping.
use super::super::*;

pub(crate) fn map_table_data_filter(filter: UiTableDataFilter) -> TableFilter {
    let value = parse_filter_value(&filter.data_type, &filter.value);
    let (op, value) = match filter.operator {
        UiTableFilterOperator::Equals => (FilterOp::Eq, value),
        UiTableFilterOperator::NotEquals => (FilterOp::Neq, value),
        UiTableFilterOperator::Contains => (FilterOp::Like, CellValue::Text(format!("%{}%", filter.value))),
        UiTableFilterOperator::StartsWith => (FilterOp::Like, CellValue::Text(format!("{}%", filter.value))),
        UiTableFilterOperator::EndsWith => (FilterOp::Like, CellValue::Text(format!("%{}", filter.value))),
        UiTableFilterOperator::GreaterThan => (FilterOp::Gt, value),
        UiTableFilterOperator::GreaterThanOrEqual => (FilterOp::Gte, value),
        UiTableFilterOperator::LessThan => (FilterOp::Lt, value),
        UiTableFilterOperator::LessThanOrEqual => (FilterOp::Lte, value),
        UiTableFilterOperator::IsNull => (FilterOp::IsNull, CellValue::Null),
        UiTableFilterOperator::IsNotNull => (FilterOp::IsNotNull, CellValue::Null),
    };
    TableFilter {
        column: filter.column,
        op,
        value,
    }
}

pub(crate) fn parse_filter_value(data_type: &str, value: &str) -> CellValue {
    let normalized = data_type.to_ascii_lowercase();
    if normalized.contains("bool") {
        return value
            .parse::<bool>()
            .map(CellValue::Bool)
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized.contains("int") || normalized.contains("serial") {
        return value
            .parse::<i64>()
            .map(CellValue::Int64)
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized.contains("real") || normalized.contains("float") || normalized.contains("double") {
        return value
            .parse::<f64>()
            .map(CellValue::Float64)
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized == "numeric"
        || normalized.starts_with("numeric(")
        || normalized == "decimal"
        || normalized.starts_with("decimal(")
    {
        return CellValue::Decimal(value.to_owned());
    }
    if normalized.contains("uuid") {
        return uuid::Uuid::parse_str(value)
            .map(|uuid| CellValue::Uuid(uuid.to_string()))
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized.contains("timestamptz") || normalized.contains("timestamp with time zone") {
        return chrono::DateTime::parse_from_rfc3339(value)
            .map(|_| CellValue::DateTime(value.to_owned()))
            .unwrap_or_else(|_| CellValue::Text(value.to_owned()));
    }
    if normalized.contains("timestamp") {
        return CellValue::DateTime(value.to_owned());
    }
    if normalized == "date" {
        return CellValue::Date(value.to_owned());
    }
    if normalized.starts_with("time") {
        return CellValue::Time(value.to_owned());
    }
    CellValue::Text(value.to_owned())
}

pub(crate) fn map_table_data_sort(sort: UiTableDataSort) -> SortClause {
    SortClause {
        column: sort.column,
        direction: if sort.descending { SortDir::Desc } else { SortDir::Asc },
    }
}

pub(crate) fn map_cell(cell: db_pro_core::domain::query::CellValue) -> UiCell {
    use db_pro_core::domain::query::CellValue;

    match cell {
        CellValue::Null => UiCell::Null,
        CellValue::Bool(value) => UiCell::Boolean(value),
        CellValue::Int64(value) => UiCell::Number(value.to_string()),
        CellValue::Float64(value) => UiCell::Number(value.to_string()),
        CellValue::Decimal(value) => UiCell::Number(value),
        CellValue::Text(value)
        | CellValue::Uuid(value)
        | CellValue::DateTime(value)
        | CellValue::Timestamp(value)
        | CellValue::TimestampTz(value)
        | CellValue::TimeTz(value)
        | CellValue::Date(value)
        | CellValue::Time(value)
        | CellValue::Interval(value)
        | CellValue::Inet(value) => UiCell::Text(value),
        CellValue::Bytes(value) => UiCell::Bytes(format!("\\x{}", hex_encode(&value))),
        CellValue::Json(value) => UiCell::Json(value.to_string()),
    }
}

pub(crate) fn ui_cell_to_domain(cell: UiCell) -> Option<CellValue> {
    match cell {
        UiCell::Null => Some(CellValue::Null),
        UiCell::Boolean(value) => Some(CellValue::Bool(value)),
        UiCell::Number(value) => value
            .parse::<i64>()
            .map(CellValue::Int64)
            .or_else(|_| value.parse::<f64>().map(CellValue::Float64))
            .ok()
            .or(Some(CellValue::Text(value))),
        UiCell::Text(value) => {
            if uuid::Uuid::parse_str(&value).is_ok() {
                Some(CellValue::Uuid(value))
            } else if chrono::DateTime::parse_from_rfc3339(&value).is_ok()
                || chrono::DateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S%z").is_ok()
                || chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S%.f").is_ok()
                || chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S").is_ok()
                || chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
                || chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S").is_ok()
            {
                Some(CellValue::DateTime(value))
            } else if chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d").is_ok() {
                Some(CellValue::Date(value))
            } else if chrono::NaiveTime::parse_from_str(&value, "%H:%M:%S").is_ok()
                || chrono::NaiveTime::parse_from_str(&value, "%H:%M:%S%.f").is_ok()
            {
                Some(CellValue::Time(value))
            } else {
                Some(CellValue::Text(value))
            }
        }
        UiCell::Json(value) => serde_json::from_str(&value).ok().map(CellValue::Json),
        UiCell::Bytes(value) => {
            let value = value.strip_prefix("\\x").unwrap_or(&value);
            if value.len() % 2 != 0 {
                return None;
            }
            (0..value.len())
                .step_by(2)
                .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
                .collect::<Option<Vec<_>>>()
                .map(CellValue::Bytes)
        }
    }
}

pub(crate) fn ui_cell_to_domain_typed(cell: UiCell, data_type: &str) -> Option<CellValue> {
    let normalized = data_type.to_ascii_lowercase();
    match cell {
        UiCell::Null => Some(CellValue::Null),
        UiCell::Number(value)
            if normalized == "numeric"
                || normalized.starts_with("numeric(")
                || normalized == "decimal"
                || normalized.starts_with("decimal(") =>
        {
            Some(CellValue::Decimal(value))
        }
        UiCell::Number(value)
            if normalized.contains("real") || normalized.contains("float") || normalized.contains("double") =>
        {
            value.parse::<f64>().ok().map(CellValue::Float64)
        }
        UiCell::Number(value) => value.parse::<i64>().ok().map(CellValue::Int64),
        UiCell::Boolean(value) => Some(CellValue::Bool(value)),
        UiCell::Text(value) if normalized.contains("uuid") => uuid::Uuid::parse_str(&value)
            .ok()
            .map(|uuid| CellValue::Uuid(uuid.to_string())),
        UiCell::Text(value)
            if normalized.contains("timestamptz") || normalized.contains("timestamp with time zone") =>
        {
            chrono::DateTime::parse_from_rfc3339(&value)
                .ok()
                .map(|_| CellValue::DateTime(value))
        }
        UiCell::Text(value) if normalized.contains("timestamp") => Some(CellValue::DateTime(value)),
        UiCell::Text(value) if normalized == "date" => Some(CellValue::Date(value)),
        UiCell::Text(value) if normalized.starts_with("time") => Some(CellValue::Time(value)),
        other => ui_cell_to_domain(other),
    }
}

pub(crate) fn map_table_info(info: db_pro_core::domain::schema::TableInfo) -> UiTableInfo {
    UiTableInfo {
        schema: info.table.schema,
        name: info.table.name,
        row_count: info.table.row_count,
        columns: info
            .columns
            .into_iter()
            .map(|column| UiTableColumn {
                name: column.name,
                data_type: column.data_type,
                ordinal: column.ordinal,
                nullable: column.nullable,
                default: column.default,
                is_primary_key: column.is_primary_key,
                is_unique: column.is_unique,
                is_identity: column.is_identity,
                is_generated: column.is_generated,
                collation: column.collation,
            })
            .collect(),
        primary_key: info.primary_key.map(|primary_key| primary_key.columns),
        indexes: info
            .indexes
            .into_iter()
            .map(|index| UiTableIndex {
                name: index.name,
                columns: index.columns,
                unique: index.unique,
                method: index.method,
                primary: index.primary,
                include_columns: index.include_columns,
                predicate: index.predicate,
                definition: index.definition,
            })
            .collect(),
        foreign_keys: info
            .foreign_keys
            .into_iter()
            .map(|foreign_key| UiTableForeignKey {
                name: foreign_key.name,
                from_columns: foreign_key.from_columns,
                to_schema: foreign_key.to_schema,
                to_table: foreign_key.to_table,
                to_columns: foreign_key.to_columns,
                on_update: foreign_key.on_update,
                on_delete: foreign_key.on_delete,
                match_option: foreign_key.match_option,
                deferrable: foreign_key.deferrable,
                initially_deferred: foreign_key.initially_deferred,
            })
            .collect(),
        check_constraints: info
            .check_constraints
            .into_iter()
            .map(|c| UiCheckConstraint {
                name: c.name,
                definition: c.definition,
            })
            .collect(),
        dependencies: info
            .dependencies
            .into_iter()
            .map(|d| UiTableDependency {
                name: d.name,
                schema: d.schema,
                kind: match d.kind {
                    db_pro_core::domain::schema::DependencyKind::Table => UiDependencyKind::Table,
                    db_pro_core::domain::schema::DependencyKind::View => UiDependencyKind::View,
                    db_pro_core::domain::schema::DependencyKind::ForeignKey => UiDependencyKind::ForeignKey,
                    db_pro_core::domain::schema::DependencyKind::Trigger => UiDependencyKind::Trigger,
                    db_pro_core::domain::schema::DependencyKind::Function => UiDependencyKind::Function,
                    db_pro_core::domain::schema::DependencyKind::Sequence => UiDependencyKind::Sequence,
                },
                direction: match d.direction {
                    db_pro_core::domain::schema::DependencyDirection::DependsOn => UiDependencyDirection::DependsOn,
                    db_pro_core::domain::schema::DependencyDirection::DependedBy => UiDependencyDirection::DependedBy,
                },
                details: d.details,
            })
            .collect(),
    }
}

pub(crate) fn map_query_result(result: db_pro_core::domain::query::QueryResult) -> UiQueryResult {
    UiQueryResult {
        row_count: result.row_count,
        duration_ms: result.duration_ms,
        columns: result
            .columns
            .into_iter()
            .map(|column| UiColumn {
                name: column.name,
                data_type: column.data_type,
                nullable: column.nullable,
            })
            .collect(),
        rows: result
            .rows
            .into_iter()
            .map(|row| row.0.into_iter().map(map_cell).collect())
            .collect(),
    }
}

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn ui_request_id(request_id: RuntimeRequestId) -> db_pro_ui::RequestId {
    db_pro_ui::RequestId(request_id.0)
}

pub(crate) fn runtime_request_id(id: RequestId) -> RuntimeRequestId {
    RuntimeRequestId(id.0)
}
