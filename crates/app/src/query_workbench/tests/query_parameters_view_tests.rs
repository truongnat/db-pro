//! Tests for `../query_parameters_view.rs` (moved out of the source file).
    use super::*;
    use crate::query::QueryDocument;

    #[test]
    fn opening_the_panel_does_not_store_an_empty_parameter_value() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "SELECT :id"));
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = crate::test_frame::frame(&ctx, Default::default(), |ui| {
            let mut context = QueryParametersContext {
                theme: DbProTheme::default(),
                session: &mut session,
                supports_parameters: true,
                driver: "PostgreSQL",
            };
            draw_parameters_panel(&mut context, ui);
        });
        let document = session.active_document().expect("document");
        assert!(
            document.parameter_values.is_empty(),
            "opening the panel must not insert an empty value, got {:?}",
            document.parameter_values
        );
        assert!(document.parameter_secrets.is_empty());
    }