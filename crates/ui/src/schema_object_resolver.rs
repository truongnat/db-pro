//! Pure schema-object read-model resolution for the object workspace.

use super::*;
use lucide_icons::Icon;

pub(super) struct SchemaObjectDetails {
    pub(super) icon: Icon,
    pub(super) kind: String,
    pub(super) name: String,
    pub(super) schema: String,
    pub(super) definition: String,
    pub(super) metadata: Option<String>,
    pub(super) query: String,
}

pub(super) fn resolve_schema_object(
    schema: &UiSchemaSummary,
    selection: &SchemaObjectSelection,
) -> Option<SchemaObjectDetails> {
    match selection {
        SchemaObjectSelection::View(name) => {
            let view = schema.views.iter().find(|view| &view.name == name)?.clone();
            Some(SchemaObjectDetails {
                icon: Icon::Eye,
                kind: "VIEW".to_owned(),
                name: view.name.clone(),
                schema: view.schema.clone(),
                definition: view.definition,
                metadata: None,
                query: format!("SELECT *\nFROM \"{}\".\"{}\"\nLIMIT 100;", view.schema, view.name),
            })
        }
        SchemaObjectSelection::Trigger(name) => {
            let trigger = schema.triggers.iter().find(|trigger| &trigger.name == name)?.clone();
            Some(SchemaObjectDetails {
                icon: Icon::Zap,
                kind: "TRIGGER".to_owned(),
                name: trigger.name,
                schema: trigger.schema,
                definition: trigger.definition.clone(),
                metadata: Some(format!(
                    "{} · {} · {}",
                    trigger.table_name, trigger.timing, trigger.event
                )),
                query: trigger.definition,
            })
        }
        SchemaObjectSelection::Function {
            name,
            identity_arguments,
        } => {
            let function = schema
                .functions
                .iter()
                .find(|function| &function.name == name && &function.identity_arguments == identity_arguments)?
                .clone();
            let display_name = if function.identity_arguments.is_empty() {
                function.name.clone()
            } else {
                format!("{}({})", function.name, function.identity_arguments)
            };
            Some(SchemaObjectDetails {
                icon: Icon::Code2,
                kind: function.routine_type,
                name: display_name,
                schema: function.schema,
                definition: function.definition.clone(),
                metadata: Some(format!(
                    "{} · {} · returns {}{}",
                    function.language,
                    function.volatility,
                    function.data_type,
                    if function.security_definer {
                        " · SECURITY DEFINER"
                    } else {
                        ""
                    }
                )),
                query: function.definition,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_view_details_and_quoted_query() {
        let schema = UiSchemaSummary {
            views: vec![UiViewSummary {
                schema: "public".into(),
                name: "order\"summary".into(),
                definition: "SELECT 1".into(),
            }],
            ..UiSchemaSummary::default()
        };

        let details = resolve_schema_object(&schema, &SchemaObjectSelection::View("order\"summary".into()))
            .expect("view should resolve");

        assert_eq!(details.kind, "VIEW");
        assert_eq!(
            details.query,
            "SELECT *\nFROM \"public\".\"order\"summary\"\nLIMIT 100;"
        );
    }

    #[test]
    fn resolves_function_display_name_and_security_metadata() {
        let schema = UiSchemaSummary {
            functions: vec![UiFunctionSummary {
                schema: "public".into(),
                name: "calculate".into(),
                routine_type: "FUNCTION".into(),
                data_type: "integer".into(),
                definition: "RETURN 1".into(),
                identity_arguments: "value integer".into(),
                language: "plpgsql".into(),
                volatility: "VOLATILE".into(),
                security_definer: true,
                parameters: Vec::new(),
            }],
            ..UiSchemaSummary::default()
        };

        let details = resolve_schema_object(
            &schema,
            &SchemaObjectSelection::Function {
                name: "calculate".into(),
                identity_arguments: "value integer".into(),
            },
        )
        .expect("function should resolve");

        assert_eq!(details.name, "calculate(value integer)");
        assert!(details
            .metadata
            .expect("metadata should exist")
            .contains("SECURITY DEFINER"));
    }

    #[test]
    fn missing_selection_returns_none() {
        assert!(resolve_schema_object(
            &UiSchemaSummary::default(),
            &SchemaObjectSelection::Trigger("missing".into())
        )
        .is_none());
    }
}
