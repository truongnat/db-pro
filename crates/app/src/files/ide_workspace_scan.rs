use super::ide_workspace_types::{WorkspaceFileNode, WorkspaceIndexEntry, WORKSPACE_MAX_DEPTH, WORKSPACE_MAX_ENTRIES};
use std::path::Path;

const INDEXABLE_EXTENSIONS: &[&str] = &["sql", "md", "txt", "toml", "json", "yml", "yaml", "dbpro-nb"];

fn is_indexable(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
        INDEXABLE_EXTENSIONS
            .iter()
            .any(|allowed| ext.eq_ignore_ascii_case(allowed))
    })
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | ".idea" | ".vscode" | "dist" | "build" | ".next"
    )
}

pub(super) fn scan_directory(
    root: &Path,
    dir: &Path,
    depth: usize,
    count: &mut usize,
) -> Result<Vec<WorkspaceFileNode>, String> {
    if depth > WORKSPACE_MAX_DEPTH || *count >= WORKSPACE_MAX_ENTRIES {
        return Ok(Vec::new());
    }
    let entries = std::fs::read_dir(dir).map_err(|error| format!("read {}: {error}", dir.display()))?;
    let mut nodes = Vec::new();
    let mut collected = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("read {}: {error}", dir.display()))?;
        collected.push(entry);
    }
    collected.sort_by_key(|entry| {
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        (!is_dir, entry.file_name().to_string_lossy().to_lowercase())
    });

    for entry in collected {
        if *count >= WORKSPACE_MAX_ENTRIES {
            break;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') && name != ".env.example" && name != ".dbpro" && name != ".sqlfluff" {
            continue;
        }
        let path = entry.path();
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        if is_dir {
            if should_skip_dir(&name) {
                continue;
            }
            *count += 1;
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let children = scan_directory(root, &path, depth + 1, count)?;
            nodes.push(WorkspaceFileNode {
                name,
                relative_path: relative,
                absolute_path: path,
                is_dir: true,
                children,
            });
        } else if is_indexable(&path) {
            *count += 1;
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            nodes.push(WorkspaceFileNode {
                name,
                relative_path: relative,
                absolute_path: path,
                is_dir: false,
                children: Vec::new(),
            });
        }
    }
    Ok(nodes)
}

pub(super) fn flatten_index(root_id: &str, nodes: &[WorkspaceFileNode]) -> Vec<WorkspaceIndexEntry> {
    let mut out = Vec::new();
    fn walk(root_id: &str, nodes: &[WorkspaceFileNode], out: &mut Vec<WorkspaceIndexEntry>) {
        for node in nodes {
            if node.is_dir {
                walk(root_id, &node.children, out);
            } else {
                out.push(WorkspaceIndexEntry {
                    root_id: root_id.to_owned(),
                    is_sql: node
                        .relative_path
                        .rsplit('.')
                        .next()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("sql")),
                    relative_path: node.relative_path.clone(),
                });
            }
        }
    }
    walk(root_id, nodes, &mut out);
    out
}
