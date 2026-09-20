use super::*;

impl DbProApp {
    pub(super) fn quick_open_items() -> Vec<PaletteItem> {
        let mut items = Self::quick_open_navigation_items();
        items.extend(Self::quick_open_schema_items());
        items.extend(Self::quick_open_activity_items());
        items
    }

    pub(super) fn command_items() -> Vec<PaletteItem> {
        let mut items = Self::query_command_items();
        items.extend(Self::workspace_command_items());
        items
    }

    fn quick_open_navigation_items() -> Vec<PaletteItem> {
        vec![
            Self::catalog_item(
                Icon::House,
                "Welcome",
                "Database workspace home",
                None,
                PaletteAction::Welcome,
            ),
            Self::catalog_item(
                Icon::FileCode2,
                "Query",
                "Open the SQL editor",
                Some(format!("{}K", Self::primary_modifier_label())),
                PaletteAction::Query,
            ),
            Self::catalog_item(
                Icon::History,
                "Query history",
                "Browse saved and recent queries",
                None,
                PaletteAction::History,
            ),
            Self::catalog_item(
                Icon::Table2,
                "Data",
                "Recent and pinned tables",
                None,
                PaletteAction::Data,
            ),
            Self::catalog_item(
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
            Self::catalog_item(
                Icon::ArrowRightLeft,
                "ER diagram",
                "Explore tables and relationships",
                None,
                PaletteAction::Diagram,
            ),
            Self::catalog_item(
                Icon::Boxes,
                "Schema workbench",
                "Create and alter schema objects",
                None,
                PaletteAction::SchemaWorkbench,
            ),
            Self::catalog_item(
                Icon::GitCompare,
                "Schema compare",
                "Diff two schema snapshots",
                None,
                PaletteAction::SchemaCompare,
            ),
            Self::catalog_item(
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
            Self::catalog_item(
                Icon::Gauge,
                "Monitor",
                "Connection health and recent statements",
                None,
                PaletteAction::Monitor,
            ),
            Self::catalog_item(
                Icon::Settings2,
                "Settings",
                "Connections, backups and restore",
                None,
                PaletteAction::Settings,
            ),
            Self::catalog_item(
                Icon::Bot,
                "Agent",
                "Open the database copilot",
                None,
                PaletteAction::Agent,
            ),
            Self::catalog_item(
                Icon::TriangleAlert,
                "Problems",
                "Open SQL diagnostics across documents",
                None,
                PaletteAction::Problems,
            ),
            Self::catalog_item(
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
            Self::catalog_item(
                Icon::Plus,
                "New query",
                "Create a fresh SQL document",
                None,
                PaletteAction::NewQuery,
            ),
            Self::catalog_item(
                Icon::Play,
                "Run query",
                "Execute the current SQL or selection",
                Some(format!("{}↵", Self::primary_modifier_label())),
                PaletteAction::RunQuery,
            ),
            Self::catalog_item(
                Icon::WandSparkles,
                "Format SQL",
                "Format the active SQL document",
                None,
                PaletteAction::FormatSql,
            ),
            Self::catalog_item(
                Icon::Database,
                "New connection",
                "Add a PostgreSQL or SQLite connection",
                None,
                PaletteAction::NewConnection,
            ),
            Self::catalog_item(
                Icon::RotateCcw,
                "Refresh schema",
                "Reload tables, views and relationships",
                None,
                PaletteAction::RefreshSchema,
            ),
            Self::catalog_item(
                Icon::Pin,
                "Pin / unpin selected table",
                "Toggle the active table in pinned Quick Open entries",
                None,
                PaletteAction::TogglePinTable(String::new()),
            ),
        ]
    }

    fn workspace_command_items() -> Vec<PaletteItem> {
        let mut items = Self::workspace_navigation_items();
        items.extend(Self::workspace_analysis_items());
        items
    }

    fn workspace_navigation_items() -> Vec<PaletteItem> {
        vec![
            Self::catalog_item(
                Icon::FolderOpen,
                "Open Folder…",
                "Open a local workspace folder (#261)",
                None,
                PaletteAction::OpenWorkspaceFolder,
            ),
            Self::catalog_item(
                Icon::Folder,
                "Close Workspace",
                "Close the active workspace folder",
                None,
                PaletteAction::CloseWorkspaceFolder,
            ),
            Self::catalog_item(
                Icon::PanelLeft,
                "Toggle explorer",
                "Show or hide the connection sidebar",
                Some(format!("{}B", Self::primary_modifier_label())),
                PaletteAction::ToggleExplorer,
            ),
            Self::catalog_item(
                Icon::Bot,
                "Open Agent",
                "Ask Agent about the active schema",
                None,
                PaletteAction::Agent,
            ),
            Self::catalog_item(
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
            Self::catalog_item(
                Icon::ChartNoAxesCombined,
                "Explain query",
                "Inspect a read-only query plan",
                None,
                PaletteAction::ExplainQuery,
            ),
            Self::catalog_item(
                Icon::Download,
                "Export results",
                "Open export options for the current result",
                None,
                PaletteAction::ExportResults,
            ),
            Self::catalog_item(
                Icon::Palette,
                "Open Component Gallery",
                "Preview DB Pro common UI design system",
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
}
