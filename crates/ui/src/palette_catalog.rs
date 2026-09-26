use super::*;

pub(super) fn quick_open_items() -> Vec<PaletteItem> {
    let mut items = quick_open_navigation_items();
    items.extend(quick_open_schema_items());
    items.extend(quick_open_activity_items());
    items
}

pub(super) fn command_items() -> Vec<PaletteItem> {
    let mut items = query_command_items();
    items.extend(workspace_command_items());
    items
}

fn quick_open_navigation_items() -> Vec<PaletteItem> {
    vec![
        catalog_item(
            Icon::House,
            "Welcome",
            "Database workspace home",
            None,
            PaletteAction::Welcome,
        ),
        catalog_item(
            Icon::FileCode2,
            "Query",
            "Open the SQL editor",
            Some(format!("{}K", primary_modifier_label())),
            PaletteAction::Query,
        ),
        catalog_item(
            Icon::History,
            "Query history",
            "Browse saved and recent queries",
            None,
            PaletteAction::History,
        ),
        catalog_item(
            Icon::Table2,
            "Data",
            "Recent and pinned tables",
            None,
            PaletteAction::Data,
        ),
        catalog_item(
            Icon::FolderOpen,
            "Files",
            "Workspace folder and project SQL files",
            None,
            PaletteAction::Files,
        ),
    ]
}

fn quick_open_schema_items() -> Vec<PaletteItem> {
    vec![
        catalog_item(
            Icon::ArrowRightLeft,
            "ER diagram",
            "Explore tables and relationships",
            None,
            PaletteAction::Diagram,
        ),
        catalog_item(
            Icon::Boxes,
            "Schema workbench",
            "Create and alter schema objects",
            None,
            PaletteAction::SchemaWorkbench,
        ),
        catalog_item(
            Icon::GitCompare,
            "Schema compare",
            "Diff two schema snapshots",
            None,
            PaletteAction::SchemaCompare,
        ),
        catalog_item(
            Icon::Upload,
            "Transfers",
            "Import, export, and background copy jobs",
            None,
            PaletteAction::Transfers,
        ),
    ]
}

fn quick_open_activity_items() -> Vec<PaletteItem> {
    vec![
        catalog_item(
            Icon::Gauge,
            "Monitor",
            "Connection health and recent statements",
            None,
            PaletteAction::Monitor,
        ),
        catalog_item(
            Icon::Settings2,
            "Settings",
            "Connections, backups and restore",
            None,
            PaletteAction::Settings,
        ),
        catalog_item(
            Icon::Bot,
            "Agent",
            "Open the database copilot",
            None,
            PaletteAction::Agent,
        ),
        catalog_item(
            Icon::TriangleAlert,
            "Problems",
            "Open SQL diagnostics across documents",
            None,
            PaletteAction::Problems,
        ),
        catalog_item(
            Icon::Activity,
            "Diagnostics",
            "App version, drivers, and redacted support summary",
            None,
            PaletteAction::Diagnostics,
        ),
    ]
}

fn query_command_items() -> Vec<PaletteItem> {
    vec![
        catalog_item(
            Icon::Plus,
            "New query",
            "Create a fresh SQL document",
            None,
            PaletteAction::NewQuery,
        ),
        catalog_item(
            Icon::Play,
            "Run query",
            "Execute the current SQL or selection",
            Some(format!("{}↵", primary_modifier_label())),
            PaletteAction::RunQuery,
        ),
        catalog_item(
            Icon::WandSparkles,
            "Format SQL",
            "Format the active SQL document",
            None,
            PaletteAction::FormatSql,
        ),
        catalog_item(
            Icon::Database,
            "New connection",
            "Add a PostgreSQL or SQLite connection",
            None,
            PaletteAction::NewConnection,
        ),
        catalog_item(
            Icon::RotateCcw,
            "Refresh schema",
            "Reload tables, views and relationships",
            None,
            PaletteAction::RefreshSchema,
        ),
        catalog_item(
            Icon::Pin,
            "Pin / unpin selected table",
            "Toggle the active table in pinned Quick Open entries",
            None,
            PaletteAction::TogglePinTable(String::new()),
        ),
    ]
}

fn workspace_command_items() -> Vec<PaletteItem> {
    let mut items = workspace_navigation_items();
    items.extend(workspace_analysis_items());
    items
}

fn workspace_navigation_items() -> Vec<PaletteItem> {
    vec![
        catalog_item(
            Icon::FolderOpen,
            "Open Folder…",
            "Open a local workspace folder (#261)",
            None,
            PaletteAction::OpenWorkspaceFolder,
        ),
        catalog_item(
            Icon::Folder,
            "Close Workspace",
            "Close the active workspace folder",
            None,
            PaletteAction::CloseWorkspaceFolder,
        ),
        catalog_item(
            Icon::PanelLeft,
            "Toggle explorer",
            "Show or hide the connection sidebar",
            Some(format!("{}B", primary_modifier_label())),
            PaletteAction::ToggleExplorer,
        ),
        catalog_item(
            Icon::Bot,
            "Open Agent",
            "Ask Agent about the active schema",
            None,
            PaletteAction::Agent,
        ),
        catalog_item(
            Icon::ArrowRightLeft,
            "Open ER diagram",
            "Show the active schema map",
            None,
            PaletteAction::Diagram,
        ),
    ]
}

fn workspace_analysis_items() -> Vec<PaletteItem> {
    vec![
        catalog_item(
            Icon::ChartNoAxesCombined,
            "Explain query",
            "Inspect a read-only query plan",
            None,
            PaletteAction::ExplainQuery,
        ),
        catalog_item(
            Icon::Download,
            "Export results",
            "Open export options for the current result",
            None,
            PaletteAction::ExportResults,
        ),
        catalog_item(
            Icon::Palette,
            "Open Component Gallery",
            "Browse DB Pro workstation primitives",
            None,
            PaletteAction::ComponentGallery,
        ),
    ]
}

fn catalog_item(
    icon: Icon,
    title: &str,
    subtitle: &str,
    shortcut: Option<String>,
    action: PaletteAction,
) -> PaletteItem {
    PaletteItem {
        icon,
        title: title.to_owned(),
        subtitle: subtitle.to_owned(),
        shortcut,
        action,
    }
}

pub(super) fn agent_action_items() -> Vec<(SearchKind, PaletteItem)> {
    vec![
        (
            SearchKind::Agent,
            PaletteItem {
                icon: Icon::Bot,
                title: "Ask Agent".to_owned(),
                subtitle: "Open the database copilot".to_owned(),
                shortcut: None,
                action: PaletteAction::Agent,
            },
        ),
        (
            SearchKind::Agent,
            PaletteItem {
                icon: Icon::Sparkles,
                title: "Explain current query".to_owned(),
                subtitle: "Agent action · explain plan".to_owned(),
                shortcut: None,
                action: PaletteAction::ExplainQuery,
            },
        ),
    ]
}
fn primary_modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    }
}
