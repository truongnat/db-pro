/// Layout width specifications for connection dialog rows.
pub struct ProfileRowWidths {
    pub name_w: f32,
    pub group_w: f32,
    pub fav_w: f32,
}

pub struct EnvRowWidths {
    pub env_w: f32,
    pub ro_w: f32,
}

pub struct ServerRowWidths {
    pub host_w: f32,
    pub port_w: f32,
}

pub struct SshRowWidths {
    pub host_w: f32,
    pub port_w: f32,
    pub user_w: f32,
    pub key_w: f32,
}

pub struct SqliteFileRowWidths {
    pub input_w: f32,
    pub button_w: f32,
}

/// Calculate 4-column card width for database engine selection.
pub fn calculate_engine_card_width(available_width: f32, gap: f32) -> f32 {
    ((available_width - 3.0 * gap) / 4.0).max(40.0)
}

/// Calculate General Profile row 1 widths (Name: 52%, Group: 36%, Favorite: 12%).
pub fn calculate_profile_row_widths(available_width: f32, gap: f32) -> ProfileRowWidths {
    let total = (available_width - 2.0 * gap).max(0.0);
    ProfileRowWidths {
        name_w: total * 0.52,
        group_w: total * 0.36,
        fav_w: total * 0.12,
    }
}

/// Calculate Environment row widths (Env: 55%, Safety: 45%).
pub fn calculate_env_row_widths(available_width: f32, gap: f32) -> EnvRowWidths {
    let total = (available_width - gap).max(0.0);
    EnvRowWidths {
        env_w: total * 0.55,
        ro_w: total * 0.45,
    }
}

/// Calculate Server row widths (Host: 68%, Port: 32%).
pub fn calculate_server_row_widths(available_width: f32, gap: f32) -> ServerRowWidths {
    let total = (available_width - gap).max(0.0);
    ServerRowWidths {
        host_w: total * 0.68,
        port_w: total * 0.32,
    }
}

/// Calculate 2 equal columns width (50% each).
pub fn calculate_half_row_width(available_width: f32, gap: f32) -> f32 {
    ((available_width - gap) * 0.5).max(0.0)
}

/// Calculate SSH tunnel row widths (Host: 38%, Port: 14%, User: 20%, Key: 28%).
pub fn calculate_ssh_row_widths(available_width: f32, gap: f32) -> SshRowWidths {
    let total = (available_width - 3.0 * gap).max(0.0);
    SshRowWidths {
        host_w: total * 0.38,
        port_w: total * 0.14,
        user_w: total * 0.20,
        key_w: total * 0.28,
    }
}

/// Calculate SQLite file picker row widths (Browse button: 110px).
pub fn calculate_sqlite_file_row_widths(available_width: f32, gap: f32) -> SqliteFileRowWidths {
    let button_w = 110.0;
    let input_w = (available_width - button_w - gap).max(120.0);
    SqliteFileRowWidths { input_w, button_w }
}
