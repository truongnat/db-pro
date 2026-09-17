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
