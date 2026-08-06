use tauri::State;

use db_pro_core::application::BackupService;
use db_pro_core::domain::backup::{BackupFormat, BackupOptions, RestoreOptions};

use crate::dto::{BackupOptionsDto, BackupResultDto, CommandError, RestoreOptionsDto};

#[tauri::command]
pub async fn backup_database(req: BackupOptionsDto, service: State<'_, BackupService>) -> Result<BackupResultDto, CommandError> {
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
    let result = service.backup(&options).await.map_err(CommandError::from)?;
    Ok(result.into())
}

#[tauri::command]
pub async fn restore_database(req: RestoreOptionsDto, service: State<'_, BackupService>) -> Result<(), CommandError> {
    let options = RestoreOptions {
        connection_id: req.connection_id,
        input_path: req.input_path,
        format: match req.format {
            crate::dto::BackupFormatDto::Plain => BackupFormat::Plain,
            crate::dto::BackupFormatDto::Custom => BackupFormat::Custom,
        },
    };
    service.restore(&options).await.map_err(CommandError::from)
}
