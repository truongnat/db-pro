use crate::domain::connection::DriverType;
use crate::domain::transfer::{TransferError, TransferMapping, TransferRow};

/// Column identity used when building a conversion plan (no secrets).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbColumnSpec {
    pub name: String,
    /// Provider type name as reported by introspection (e.g. `integer`, `text`, `uuid`).
    pub type_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionKind {
    Identity,
    /// Safe widening / dialect-equivalent cast.
    SafeCast,
    /// Values go through text; may be lossy for binary/exotic types.
    TextBridge,
    /// Explicitly refused — job must not start without remapping.
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnConversion {
    pub source_index: usize,
    pub source_name: String,
    pub source_type: String,
    pub target_name: String,
    pub target_type: String,
    pub kind: ConversionKind,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionPlan {
    pub source_driver: DriverType,
    pub target_driver: DriverType,
    pub columns: Vec<ColumnConversion>,
    pub warnings: Vec<String>,
    pub blocked: bool,
}

impl ConversionPlan {
    pub fn ensure_runnable(&self) -> Result<(), TransferError> {
        if self.blocked {
            let detail = self
                .columns
                .iter()
                .filter(|c| c.kind == ConversionKind::Unsupported)
                .map(|c| {
                    format!(
                        "{}:{} → {}:{} ({})",
                        c.source_name,
                        c.source_type,
                        c.target_name,
                        c.target_type,
                        c.warning.as_deref().unwrap_or("unsupported")
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            return Err(TransferError::Conversion(format!(
                "transfer blocked by unsupported conversions: {detail}"
            )));
        }
        Ok(())
    }
}

/// Normalize introspection type names for comparison.
fn normalize_type(raw: &str) -> String {
    raw.trim().to_ascii_lowercase().replace("character varying", "varchar")
}

fn classify_pair(
    source_driver: DriverType,
    target_driver: DriverType,
    source_ty: &str,
    target_ty: &str,
) -> (ConversionKind, Option<String>) {
    let src = normalize_type(source_ty);
    let dst = normalize_type(target_ty);

    if source_driver == target_driver {
        if src == dst || type_aliases(&src, &dst) {
            return (ConversionKind::Identity, None);
        }
        if same_provider_safe_cast(&src, &dst) {
            return (ConversionKind::SafeCast, Some(format!("cast {src} → {dst}")));
        }
        return (
            ConversionKind::Unsupported,
            Some(format!("same-provider type mismatch {src} → {dst}")),
        );
    }

    // Explicitly supported cross-provider combinations only.
    match (source_driver, target_driver) {
        (DriverType::Postgres, DriverType::SQLite) | (DriverType::SQLite, DriverType::Postgres) => {
            pg_sqlite_conversion(&src, &dst)
        }
        (DriverType::Mysql, _) | (_, DriverType::Mysql) => (
            ConversionKind::Unsupported,
            Some("MySQL database-to-database transfer is not enabled yet".into()),
        ),
        _ => (
            ConversionKind::Unsupported,
            Some(format!(
                "no conversion matrix for {source_driver:?} → {target_driver:?}"
            )),
        ),
    }
}

fn type_aliases(a: &str, b: &str) -> bool {
    matches!(
        (a, b),
        ("int", "integer")
            | ("integer", "int")
            | ("int4", "integer")
            | ("integer", "int4")
            | ("int8", "bigint")
            | ("bigint", "int8")
            | ("bool", "boolean")
            | ("boolean", "bool")
            | ("varchar", "text")
            | ("text", "varchar")
            | ("character varying", "text")
    )
}

fn same_provider_safe_cast(src: &str, dst: &str) -> bool {
    matches!(
        (src, dst),
        ("smallint", "integer")
            | ("integer", "bigint")
            | ("real", "double precision")
            | ("float", "double")
            | ("varchar", "text")
            | ("char", "text")
            | ("text", "varchar")
    )
}

fn pg_sqlite_conversion(src: &str, dst: &str) -> (ConversionKind, Option<String>) {
    if let Some(kind) = pg_sqlite_known_conversion(src, dst) {
        let warning = match kind {
            ConversionKind::TextBridge => Some(format!(
                "PG↔SQLite bridge {src} → {dst} via text/affinity; verify values before production use"
            )),
            ConversionKind::SafeCast => Some(format!("PG↔SQLite cast {src} → {dst}")),
            ConversionKind::Identity | ConversionKind::Unsupported => None,
        };
        return (kind, warning);
    }

    if matches!(src, "array" | "tsvector" | "xml" | "inet" | "cidr" | "macaddr") || src.contains("[]") {
        return (
            ConversionKind::Unsupported,
            Some(format!("PostgreSQL type `{src}` has no SQLite mapping")),
        );
    }

    (
        ConversionKind::Unsupported,
        Some(format!("no explicit PG↔SQLite mapping for {src} → {dst}")),
    )
}

fn pg_sqlite_known_conversion(src: &str, dst: &str) -> Option<ConversionKind> {
    if src == dst || type_aliases(src, dst) {
        return Some(ConversionKind::Identity);
    }
    if pg_sqlite_text_bridge_pair(src, dst) {
        return Some(ConversionKind::TextBridge);
    }
    pg_sqlite_affinity_pair(src, dst).then_some(ConversionKind::SafeCast)
}

fn pg_sqlite_text_bridge_pair(src: &str, dst: &str) -> bool {
    matches!(
        (src, dst),
        ("uuid", "text")
            | ("json", "text")
            | ("jsonb", "text")
            | ("numeric", "text")
            | ("timestamp", "text")
            | ("timestamptz", "text")
            | ("date", "text")
            | ("text", "uuid")
            | ("text", "json")
            | ("text", "jsonb")
            | ("text", "numeric")
            | ("text", "timestamp")
            | ("text", "date")
            | ("boolean", "integer")
            | ("bool", "integer")
            | ("integer", "boolean")
    )
}

fn pg_sqlite_affinity_pair(src: &str, dst: &str) -> bool {
    matches!(
        (src, dst),
        ("integer", "integer")
            | ("integer", "int")
            | ("int", "integer")
            | ("bigint", "integer")
            | ("text", "text")
            | ("varchar", "text")
            | ("text", "varchar")
            | ("real", "real")
            | ("double precision", "real")
            | ("boolean", "integer")
            | ("bool", "integer")
            | ("integer", "boolean")
            | ("blob", "bytea")
            | ("bytea", "blob")
            | ("numeric", "text")
            | ("text", "numeric")
            | ("uuid", "text")
            | ("text", "uuid")
            | ("json", "text")
            | ("jsonb", "text")
            | ("text", "json")
            | ("text", "jsonb")
            | ("timestamp", "text")
            | ("timestamptz", "text")
            | ("text", "timestamp")
            | ("date", "text")
            | ("text", "date")
    )
}

/// Build a previewable conversion plan. Does not write.
pub fn build_conversion_plan(
    source_driver: DriverType,
    target_driver: DriverType,
    source_columns: &[DbColumnSpec],
    target_columns: &[DbColumnSpec],
    mapping: &TransferMapping,
) -> Result<ConversionPlan, TransferError> {
    let pairs: Vec<(usize, String)> = if mapping.columns.is_empty() {
        // Auto-match by name (case-insensitive), then by position.
        let mut auto = Vec::new();
        for (idx, src) in source_columns.iter().enumerate() {
            if let Some(tgt) = target_columns.iter().find(|t| t.name.eq_ignore_ascii_case(&src.name)) {
                auto.push((idx, tgt.name.clone()));
            } else if let Some(tgt) = target_columns.get(idx) {
                auto.push((idx, tgt.name.clone()));
            }
        }
        auto
    } else {
        mapping.columns.clone()
    };

    if pairs.is_empty() {
        return Err(TransferError::Mapping(
            "no column mappings resolved for database transfer".into(),
        ));
    }

    let mut columns = Vec::with_capacity(pairs.len());
    let mut warnings = Vec::new();
    let mut blocked = false;

    for (source_index, target_name) in pairs {
        if target_name.is_empty() {
            continue;
        }
        let source = source_columns
            .get(source_index)
            .ok_or_else(|| TransferError::Mapping(format!("source column index {source_index} out of range")))?;
        let target = target_columns
            .iter()
            .find(|c| c.name == target_name)
            .ok_or_else(|| TransferError::Mapping(format!("target column `{target_name}` not found")))?;

        let (kind, warning) = classify_pair(source_driver, target_driver, &source.type_name, &target.type_name);
        if kind == ConversionKind::Unsupported {
            blocked = true;
        }
        if let Some(ref w) = warning {
            warnings.push(format!("{} → {}: {w}", source.name, target.name));
        }
        columns.push(ColumnConversion {
            source_index,
            source_name: source.name.clone(),
            source_type: source.type_name.clone(),
            target_name: target.name.clone(),
            target_type: target.type_name.clone(),
            kind,
            warning,
        });
    }

    Ok(ConversionPlan {
        source_driver,
        target_driver,
        columns,
        warnings,
        blocked,
    })
}

/// Capability gate: both endpoints must allow table data mutation/read independently.
pub fn assert_endpoint_capabilities(
    source_allows_read: bool,
    target_allows_write: bool,
    create_target_requested: bool,
    target_allows_create_table: bool,
) -> Result<(), TransferError> {
    if !source_allows_read {
        return Err(TransferError::Source(
            "source connection does not allow reading table data for transfer".into(),
        ));
    }
    if !target_allows_write {
        return Err(TransferError::Target(
            "target connection does not allow writing table data for transfer".into(),
        ));
    }
    if create_target_requested && !target_allows_create_table {
        return Err(TransferError::Target(
            "create-target-table requested but target provider/capability forbids DDL".into(),
        ));
    }
    Ok(())
}

/// Apply mapping to a source row → target-ordered cells for insert.
pub fn project_row(row: &TransferRow, plan: &ConversionPlan) -> Result<TransferRow, TransferError> {
    let mut cells = Vec::with_capacity(plan.columns.len());
    for column in &plan.columns {
        let value = row.cells.get(column.source_index).ok_or_else(|| {
            TransferError::Mapping(format!(
                "row missing source index {} for column {}",
                column.source_index, column.source_name
            ))
        })?;
        cells.push(value.clone());
    }
    Ok(TransferRow { cells })
}
