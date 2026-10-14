//! Brand icons for database engines shown in the connection dialog.
//!
//! PNGs are pre-rasterized (48×48 RGBA) from their public brand sources and
//! embedded with `include_bytes!` — no runtime decode cost beyond the first
//! upload per icon. `texture` caches one `TextureHandle` per asset per ctx.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// One embedded 48×48 brand PNG keyed by the spec's `icon_png` path.
pub struct DriverIcon {
    pub name: &'static str,
    png: &'static [u8],
}

pub const DRIVER_ICONS: &[DriverIcon] = &[
    DriverIcon { name: "postgresql", png: include_bytes!("../../assets/drivers/postgresql.png") },
    DriverIcon { name: "sqlite", png: include_bytes!("../../assets/drivers/sqlite.png") },
    DriverIcon { name: "mysql", png: include_bytes!("../../assets/drivers/mysql.png") },
    DriverIcon { name: "microsoftsqlserver", png: include_bytes!("../../assets/drivers/microsoftsqlserver.png") },
    DriverIcon { name: "mariadb", png: include_bytes!("../../assets/drivers/mariadb.png") },
    DriverIcon { name: "redis", png: include_bytes!("../../assets/drivers/redis.png") },
    DriverIcon { name: "mongodb", png: include_bytes!("../../assets/drivers/mongodb.png") },
    DriverIcon { name: "oracle", png: include_bytes!("../../assets/drivers/oracle.png") },
    DriverIcon { name: "cockroachlabs", png: include_bytes!("../../assets/drivers/cockroachlabs.png") },
    DriverIcon { name: "clickhouse", png: include_bytes!("../../assets/drivers/clickhouse.png") },
    DriverIcon { name: "elasticsearch", png: include_bytes!("../../assets/drivers/elasticsearch.png") },
    DriverIcon { name: "cassandra", png: include_bytes!("../../assets/drivers/cassandra.png") },
    DriverIcon { name: "azuresql", png: include_bytes!("../../assets/drivers/azuresql.png") },
    DriverIcon { name: "neon", png: include_bytes!("../../assets/drivers/neon.png") },
    DriverIcon { name: "supabase", png: include_bytes!("../../assets/drivers/supabase.png") },
];

static CACHE: LazyLock<Mutex<HashMap<&'static str, egui::TextureHandle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Resolve (and lazily decode) the brand icon for `icon_png` name like
/// `"postgresql"`. Returns `None` for unknown names — callers fall back to
/// the generic lucide glyph.
pub fn texture(ctx: &egui::Context, name: &'static str) -> Option<egui::TextureId> {
    let mut cache = CACHE.lock().ok()?;
    if let Some(handle) = cache.get(name) {
        return Some(handle.id());
    }
    let icon = DRIVER_ICONS.iter().find(|i| i.name == name)?;
    let decoded = image::load_from_memory_with_format(icon.png, image::ImageFormat::Png).ok()?;
    let rgba = decoded.to_rgba8();
    let (w, h) = rgba.dimensions();
    let color = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
    let handle = ctx.load_texture(name, color, egui::TextureOptions::LINEAR);
    let id = handle.id();
    cache.insert(name, handle);
    Some(id)
}
