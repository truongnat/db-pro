//! Static SQL snippets shared by the query editor, palette and sidebar.

pub(super) fn builtin_sql_snippets() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        ("SELECT table", "sel*", "SELECT *\nFROM ${1:table_name}\nLIMIT ${2:100};"),
        (
            "UPDATE by primary key",
            "upd*",
            "UPDATE ${1:table_name}\nSET ${2:column_name} = ${3:value}\nWHERE ${4:id} = ${5:1};",
        ),
        (
            "INSERT row",
            "ins*",
            "INSERT INTO ${1:table_name} (${2:column_a}, ${3:column_b})\nVALUES ($1, $2);",
        ),
        ("DELETE with WHERE", "del*", "DELETE FROM ${1:table_name}\nWHERE ${2:id} = $1;"),
        (
            "EXPLAIN ANALYZE",
            "exp*",
            "EXPLAIN (ANALYZE, BUFFERS)\nSELECT *\nFROM ${1:table_name}\nWHERE ${2:id} = $1;",
        ),
        (
            "CREATE INDEX",
            "idx*",
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_${1:table_name}_${2:column_name}\nON ${1:table_name} (${2:column_name});",
        ),
    ]
}

#[cfg(test)]
#[path = "tests/query_snippets_tests.rs"]
mod tests;

