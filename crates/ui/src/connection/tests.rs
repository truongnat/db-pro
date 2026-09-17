use super::logic::*;
use super::mapper::*;
use super::view::*;
use crate::{DbProApp, DbProTheme, UiConnectionDraft, UiDriver, UiSslMode};

fn rendered_texts(app: &mut DbProApp) -> Vec<String> {
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
    let output = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.draw_postgres_connection_fields(ui);
        });
    });

    let mut texts = Vec::new();
    for clipped in &output.shapes {
        collect(&clipped.shape, &mut texts);
    }
    texts
}

#[test]
fn ssh_section_discloses_its_unqualified_v0_1_status() {
    let mut app = DbProApp::default();
    app.connection_draft.ssh_tunnel_enabled = false;

    let texts = rendered_texts(&mut app);

    assert!(
        texts.iter().any(|text| text == SSH_QUALIFICATION_HINT),
        "the SSH section must carry the LIM-006 qualification caveat before the tunnel is enabled; painted texts: {texts:?}"
    );
}

#[test]
fn ssh_caveat_is_non_blocking_and_keeps_the_control_usable() {
    let mut app = DbProApp::default();
    app.connection_draft.ssh_tunnel_enabled = true;

    let texts = rendered_texts(&mut app);

    assert!(
        texts.iter().any(|text| text == SSH_QUALIFICATION_HINT),
        "the caveat stays visible while the tunnel is enabled; painted texts: {texts:?}"
    );
    assert!(
        texts.iter().any(|text| text.contains("SSH Host")),
        "enabling the tunnel still renders its fields — the caveat changes no behaviour; painted texts: {texts:?}"
    );
}

#[test]
fn ssh_caveat_wording_tracks_the_recorded_limitation() {
    assert!(SSH_QUALIFICATION_HINT.contains("Unqualified in v0.1"));
    assert!(SSH_QUALIFICATION_HINT.contains("not been end-to-end tested"));
    assert!(SSH_QUALIFICATION_HINT.contains("may not work reliably"));
}

#[test]
fn test_i18n_multilingual_locale_switch() {
    use crate::UiLanguage;

    for lang in UiLanguage::ALL {
        lang.apply();
        let code = rust_i18n::locale();
        assert_eq!(&*code, lang.code());

        // Verify key lookup in each language
        let test_text = t!("connection.test_connection");
        assert!(
            !test_text.is_empty(),
            "translation for connection.test_connection missing in {}",
            lang.code()
        );
    }

    // Reset back to English
    UiLanguage::English.apply();
}

#[test]
fn test_validate_connection_draft() {
    let mut draft = UiConnectionDraft {
        name: String::new(),
        ..Default::default()
    };
    assert!(validate_connection_draft(&draft).is_err());

    draft.name = "Test DB".into();
    draft.database = String::new();
    assert!(validate_connection_draft(&draft).is_err());

    draft.database = "postgres".into();
    draft.port = "5432".into();
    assert!(validate_connection_draft(&draft).is_ok());

    draft.port = "invalid".into();
    assert!(validate_connection_draft(&draft).is_err());

    draft.port = "5432".into();
    draft.ssh_tunnel_enabled = true;
    assert!(validate_connection_draft(&draft).is_err());

    draft.ssh_host = "bastion.test".into();
    draft.ssh_user = "user".into();
    draft.ssh_private_key = "/path/to/key".into();
    draft.ssh_port = "22".into();
    assert!(validate_connection_draft(&draft).is_ok());
}

#[test]
fn test_driver_switching_and_default_ports() {
    let mut draft = UiConnectionDraft::default();
    select_driver(&mut draft, UiDriver::Postgres);
    assert_eq!(draft.port, "5432");
    assert_eq!(draft.ssl_mode, UiSslMode::Require);

    select_driver(&mut draft, UiDriver::Mysql);
    assert_eq!(draft.port, "3306");
    assert_eq!(draft.ssl_mode, UiSslMode::Require);

    select_driver(&mut draft, UiDriver::SqlServer);
    assert_eq!(draft.port, "1433");
    assert_eq!(draft.ssl_mode, UiSslMode::Require);
}

#[test]
fn test_mapper_snippet_parsing() {
    let mut draft = UiConnectionDraft::default();
    let uri = "postgresql://myuser:secret@myhost:5432/mydb?sslmode=verify-full";
    assert!(apply_connection_snippet(&mut draft, uri).is_ok());
    assert_eq!(draft.username, "myuser");
    assert_eq!(draft.password, "secret");
    assert_eq!(draft.host, "myhost");
    assert_eq!(draft.port, "5432");
    assert_eq!(draft.database, "mydb");
    assert_eq!(draft.ssl_mode, UiSslMode::VerifyFull);
}

#[test]
fn test_connection_config_mappings() {
    use super::config::*;
    assert_eq!(default_port_for_driver(UiDriver::Postgres), "5432");
    assert_eq!(default_port_for_driver(UiDriver::Mysql), "3306");
    assert_eq!(default_port_for_driver(UiDriver::SqlServer), "1433");
    assert_eq!(default_database_for_driver(UiDriver::SqlServer), "master");

    assert_eq!(environment_to_index("Development"), 0);
    assert_eq!(environment_to_index("Staging"), 1);
    assert_eq!(environment_to_index("Production"), 2);
    assert_eq!(environment_to_index("Custom"), 3);
    assert_eq!(index_to_environment(2), "Production");

    assert_eq!(ssl_mode_to_index(UiSslMode::Disable), 0);
    assert_eq!(ssl_mode_to_index(UiSslMode::Require), 1);
    assert_eq!(ssl_mode_to_index(UiSslMode::VerifyCa), 2);
    assert_eq!(ssl_mode_to_index(UiSslMode::VerifyFull), 3);
    assert_eq!(index_to_ssl_mode(3), UiSslMode::VerifyFull);
}

#[test]
fn test_connection_layout_calculations() {
    use super::layout::*;
    let card_w = calculate_engine_card_width(800.0, 8.0);
    assert!((card_w - (800.0 - 24.0) / 4.0).abs() < 0.001);

    let prof = calculate_profile_row_widths(800.0, 8.0);
    let total_prof = prof.name_w + prof.group_w + prof.fav_w;
    assert!((total_prof - (800.0 - 16.0)).abs() < 0.01);

    let env = calculate_env_row_widths(800.0, 8.0);
    let total_env = env.env_w + env.ro_w;
    assert!((total_env - (800.0 - 8.0)).abs() < 0.01);

    let srv = calculate_server_row_widths(800.0, 8.0);
    let total_srv = srv.host_w + srv.port_w;
    assert!((total_srv - (800.0 - 8.0)).abs() < 0.01);

    let half = calculate_half_row_width(800.0, 8.0);
    assert_eq!(half, (800.0 - 8.0) * 0.5);

    let ssh = calculate_ssh_row_widths(800.0, 8.0);
    let total_ssh = ssh.host_w + ssh.port_w + ssh.user_w + ssh.key_w;
    assert!((total_ssh - (800.0 - 24.0)).abs() < 0.01);
}
