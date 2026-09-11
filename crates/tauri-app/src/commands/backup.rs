use std::path::Path;
use std::process::Command;
use tauri::{AppHandle, Emitter, State};

use db_pro_core::domain::backup::{BackupFormat, BackupOptions, RestoreOptions};
use db_pro_runtime::DbProRuntime;

use crate::dto::{BackupOptionsDto, BackupProgressDto, BackupResultDto, CommandError, RestoreOptionsDto};

fn emit_progress(app: &AppHandle, operation: &str, status: &str, path: &str, message: Option<String>) {
    let _ = app.emit(
        "backup-progress",
        BackupProgressDto {
            operation: operation.to_owned(),
            status: status.to_owned(),
            path: path.to_owned(),
            message,
        },
    );
}

#[tauri::command]
pub async fn backup_database(
    req: BackupOptionsDto,
    app: AppHandle,
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
) -> Result<BackupResultDto, CommandError> {
    emit_progress(&app, "backup", "started", &req.output_path, None);
    let options = BackupOptions {
        connection_id: req.connection_id,
        output_path: req.output_path,
        format: match req.format {
            crate::dto::BackupFormatDto::Plain => BackupFormat::Plain,
            crate::dto::BackupFormatDto::Custom => BackupFormat::Custom,
        },
        schemas: req.schemas.unwrap_or_default(),
        tables: req.tables.unwrap_or_default(),
    };
    match runtime.backup_api().backup(&options).await {
        Ok(result) => {
            emit_progress(&app, "backup", "completed", &options.output_path, None);
            Ok(result.into())
        }
        Err(error) => {
            let error = CommandError::from(error);
            emit_progress(
                &app,
                "backup",
                "failed",
                &options.output_path,
                Some(error.message.clone()),
            );
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn restore_database(
    req: RestoreOptionsDto,
    app: AppHandle,
    runtime: State<'_, std::sync::Arc<DbProRuntime>>,
) -> Result<(), CommandError> {
    emit_progress(&app, "restore", "started", &req.input_path, None);
    let options = RestoreOptions {
        connection_id: req.connection_id,
        input_path: req.input_path,
        format: match req.format {
            crate::dto::BackupFormatDto::Plain => BackupFormat::Plain,
            crate::dto::BackupFormatDto::Custom => BackupFormat::Custom,
        },
    };
    match runtime.backup_api().restore(&options).await {
        Ok(()) => {
            emit_progress(&app, "restore", "completed", &options.input_path, None);
            Ok(())
        }
        Err(error) => {
            let error = CommandError::from(error);
            emit_progress(
                &app,
                "restore",
                "failed",
                &options.input_path,
                Some(error.message.clone()),
            );
            Err(error)
        }
    }
}

#[tauri::command]
pub fn reveal_backup_path(path: String) -> Result<(), CommandError> {
    let path = Path::new(&path);
    let target = if path.is_dir() {
        path
    } else {
        path.parent().ok_or_else(|| CommandError {
            error: "VALIDATION_ERROR".into(),
            message: "backup path has no parent directory".into(),
            message_id: "backup.revealInvalidPath".into(),
            details: None,
            retryable: false,
        })?
    };

    if !target.is_dir() {
        return Err(CommandError {
            error: "NOT_FOUND".into(),
            message: "backup directory does not exist".into(),
            message_id: "backup.revealMissingPath".into(),
            details: None,
            retryable: false,
        });
    }

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg("-R").arg(path);
        command
    };
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer");
        command.arg(format!("/select,{}", path.display()));
        command
    };
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(target);
        command
    };

    command.spawn().map(|_| ()).map_err(|error| CommandError {
        error: "INTERNAL_ERROR".into(),
        message: format!("failed to open backup directory: {error}"),
        message_id: "backup.revealFailed".into(),
        details: None,
        retryable: true,
    })
}

#[cfg(test)]
mod tests {
    use super::reveal_backup_path;

    #[test]
    fn reveal_rejects_a_missing_directory() {
        let error = reveal_backup_path("/definitely-missing-db-pro-backup/backup.sql".into()).unwrap_err();

        assert_eq!(error.error, "NOT_FOUND");
    }
}
