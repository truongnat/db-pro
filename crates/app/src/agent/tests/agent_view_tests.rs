//! Tests for `../agent_view.rs` (moved out of the source file).
    use super::super::agent_settings_view::{AgentSettingsContext, AI_EGRESS_DISCLOSURE};
    use super::*;

    /// Every text run the frame actually painted.
    ///
    /// The assertions below read the rendered frame rather than the constant the panel is written
    /// from, so deleting the label while keeping the constant still fails (the same idiom the SSH
    /// qualification caveat is pinned with, `connection_view.rs`).
    fn rendered_settings_texts(app: &mut DbProApp) -> Vec<String> {
        fn collect(shape: &egui::Shape, texts: &mut Vec<String>) {
            match shape {
                egui::Shape::Text(text) => texts.push(text.galley.text().to_owned()),
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        collect(shape, texts);
                    }
                }
                _ => {}
            }
        }

        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let output = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut settings = AgentSettingsContext {
                    theme: app.theme,
                    provider_label: &app.agent.provider_label,
                    api_key_draft: &mut app.agent.api_key_draft,
                    api_key_show_password: &mut app.agent.api_key_show_password,
                    configure_request: app.agent.configure_request,
                    auto_run_read_only: &mut app.agent.auto_run_read_only,
                };
                let actions = settings.draw(ui);
                assert!(actions.is_empty());
            });
        });

        let mut texts = Vec::new();
        for clipped in &output.shapes {
            collect(&clipped.shape, &mut texts);
        }
        texts
    }

    #[test]
    fn agent_key_section_discloses_what_the_ai_path_sends() {
        let mut app = DbProApp::default();

        let texts = rendered_settings_texts(&mut app);

        assert!(
            texts.iter().any(|text| text == AI_EGRESS_DISCLOSURE),
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            "the surface that enables the AI provider must state what leaves the machine (#242); painted texts: {texts:?}"
        );
        // The note must not replace the key-handling sentence the section already carried.
        assert!(
            texts
                .iter()
                .any(|text| text.contains("stored in DB Pro's secure secret store")),
            "the API-key handling caption is still painted; painted texts: {texts:?}"
        );
    }

    #[test]
    fn agent_egress_disclosure_names_the_provider_payload_and_limits() {
        // What is sent, and the code-derived bounds rather than the audit's stale 20x12.
        assert!(AI_EGRESS_DISCLOSURE.contains("prompts"));
        assert!(AI_EGRESS_DISCLOSURE.contains("SQL they reference"));
        assert!(AI_EGRESS_DISCLOSURE.contains("schema names and types"));
        assert!(AI_EGRESS_DISCLOSURE.contains("up to 20 sample result rows"));
        assert!(AI_EGRESS_DISCLOSURE.contains("50 columns"));
        assert!(AI_EGRESS_DISCLOSURE.contains("256 characters"));
        // And the only-egress claim, which is what the audit verified.
        assert!(AI_EGRESS_DISCLOSURE.contains("only other outbound connections"));
        // The audit's finding was silence, so the note states the data flow and nothing more: a
        // privacy claim would contradict LIM-019's "Actual behavior" field. This is a keyword guard,
        // not a proof of semantics -- it catches the obvious overclaim, and the painted-text test
        // above pins the wording itself.
        let lowered = AI_EGRESS_DISCLOSURE.to_lowercase();
        for overclaim in ["private", "anonymous", "never leaves", "encrypted", "secure"] {
            assert!(
                !lowered.contains(overclaim),
                "the disclosure must not claim {overclaim:?}: it records what is sent, not a privacy guarantee"
            );
        }
    }