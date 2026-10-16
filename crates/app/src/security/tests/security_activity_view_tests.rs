//! Tests for `../security_activity_view.rs` (moved out of the source file).
    use super::{
        security_role_details_view::SecurityRoleDetailsAction, SecurityActivityContext, SecurityState,
    };

    #[test]
    fn failed_security_dispatch_preserves_password_draft() {
        let (mut bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        drop(command_rx);
        let mut dispatcher = crate::app::command_dispatch::RuntimeCommandDispatcher::new(&mut bridge);
        let mut state = SecurityState {
            security_password: "draft-password".to_owned(),
            ..SecurityState::default()
        };
        let mut table_state = crate::app::TableState::default();
        let mut feedback = crate::app::FeedbackState::default();

        SecurityActivityContext {
            theme: crate::DbProTheme::default(),
            state: &mut state,
            table_state: &mut table_state,
            connected: true,
            is_postgres: true,
            connection_id: Some("conn-1"),
            command_dispatcher: &mut dispatcher,
            feedback: &mut feedback,
        }
        .apply_role_details_actions(
            "app_user",
            vec![SecurityRoleDetailsAction::UpdatePassword("new-password".to_owned())],
        );

        assert_eq!(state.security_password, "draft-password");
    }

    #[test]
    fn role_attribute_and_revoke_wait_for_confirm_and_privilege_errors_are_shown() {
        let (mut bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        let mut dispatcher = crate::app::command_dispatch::RuntimeCommandDispatcher::new(&mut bridge);
        let mut state = SecurityState::default();
        let mut table_state = crate::app::TableState::default();
        let mut feedback = crate::app::FeedbackState::default();

        {
            let mut activity = SecurityActivityContext {
                theme: crate::DbProTheme::default(),
                state: &mut state,
                table_state: &mut table_state,
                connected: true,
                is_postgres: true,
                connection_id: Some("conn-1"),
                command_dispatcher: &mut dispatcher,
                feedback: &mut feedback,
            };

            activity.apply_role_details_actions(
                "analyst",
                vec![SecurityRoleDetailsAction::AlterRole(
                    db_pro_core::domain::user::RoleAttributes {
                        createdb: Some(true),
                        ..Default::default()
                    },
                )],
            );
            assert!(
                matches!(command_rx.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)),
                "enabling CREATEDB must wait for confirmation"
            );
            assert!(matches!(
                activity.state.security_pending,
                Some(super::super::security_state::PendingSecurityAction::EnableAttribute { .. })
            ));

            activity.apply_confirmation_action(
                super::super::security_confirmation_view::SecurityConfirmationAction::ConfirmPending,
            );
            match command_rx.try_recv() {
                Ok(crate::UiCommand::AlterRole { name, attributes, .. }) => {
                    assert_eq!(name, "analyst");
                    assert_eq!(attributes.createdb, Some(true));
                }
                Ok(_) => panic!("confirm did not send AlterRole"),
                Err(error) => panic!("confirm sent no command: {error}"),
            }
            assert!(activity.state.security_pending.is_none());

            activity.apply_role_details_actions(
                "analyst",
                vec![SecurityRoleDetailsAction::AlterRole(
                    db_pro_core::domain::user::RoleAttributes {
                        createdb: Some(false),
                        ..Default::default()
                    },
                )],
            );
            match command_rx.try_recv() {
                Ok(crate::UiCommand::AlterRole { attributes, .. }) => {
                    assert_eq!(attributes.createdb, Some(false));
                }
                Ok(_) => panic!("turning CREATEDB off did not send AlterRole"),
                Err(error) => panic!("turning CREATEDB off sent no command: {error}"),
            }
            assert!(activity.state.security_pending.is_none());

            activity.apply_role_details_actions(
                "analyst",
                vec![SecurityRoleDetailsAction::RevokePrivilege(
                    db_pro_core::domain::user::Privilege {
                        object_kind: db_pro_core::domain::user::PrivilegeObjectKind::Table,
                        schema: "public".to_owned(),
                        object_name: "orders".to_owned(),
                        privilege_type: "SELECT".to_owned(),
                    },
                )],
            );
            assert!(matches!(
                command_rx.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Empty)
            ));
            activity.apply_confirmation_action(
                super::super::security_confirmation_view::SecurityConfirmationAction::ConfirmPending,
            );
            match command_rx.try_recv() {
                Ok(crate::UiCommand::RevokePrivilege {
                    role_name,
                    object_name,
                    privilege,
                    ..
                }) => {
                    assert_eq!(role_name, "analyst");
                    assert_eq!(object_name, "orders");
                    assert_eq!(privilege, "SELECT");
                }
                Ok(_) => panic!("confirm did not send RevokePrivilege"),
                Err(error) => panic!("revoke confirm sent no command: {error}"),
            }

            activity.state.security_rls_restrictive = true;
            activity.state.security_rls_table = "orders".to_owned();
            activity.state.security_rls_policy_name = "tenant_iso".to_owned();
            activity.state.security_rls_using = "tenant_id = current_setting('app.tenant')".to_owned();
            activity.preview_rls_policy(db_pro_core::domain::object_mutation::ObjectAction::Create);
            let sql = activity.state.security_rls_preview_sql.as_str();
            assert!(sql.contains("AS RESTRICTIVE"), "{sql}");
            assert!(!sql.contains("AS PERMISSIVE"), "{sql}");
        }

        super::request_security_role_details(
            &mut state,
            Some("conn-1"),
            "analyst",
            &mut dispatcher,
            &mut feedback,
        );
        let mut privilege_request = None;
        while let Ok(command) = command_rx.try_recv() {
            if let crate::UiCommand::ListPrivileges {
                request_id, role_name, ..
            } = command
            {
                assert_eq!(role_name, "analyst");
                privilege_request = Some(request_id.0);
            }
        }
        let request_id = privilege_request.expect("ListPrivileges was not requested");
        assert_eq!(state.security_privilege_request, Some((request_id, "conn-1".to_owned())));
        let message = "permission denied for relation usage_privileges";
        assert!(state.note_privilege_failure(request_id, message));
        assert_eq!(state.security_error.as_deref(), Some(message));
        assert!(state.security_privileges.is_empty());

        let privileges_view = include_str!("../security_role_details_view.rs");
        let error_at = privileges_view
            .find("security_error")
            .expect("privilege view shows security_error");
        let empty_at = privileges_view
            .find("No privileges listed")
            .expect("empty privilege copy");
        assert!(error_at < empty_at);

        let manager = include_str!("../../../../infrastructure/src/postgres/user_manager.rs");
        assert!(
            !manager.contains("return Vec::new()"),
            "privilege reads must return the query error"
        );
    }