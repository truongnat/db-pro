use super::ide_workspace_types::{
    WorkspaceDiagnostic, WorkspaceDiagnosticSeverity, WorkspaceFileNode, WorkspaceIndexEntry, WORKSPACE_MAX_DEPTH,
    WORKSPACE_MAX_ENTRIES,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const INDEXABLE_EXTENSIONS: &[&str] = &["sql", "md", "txt", "toml", "json", "yml", "yaml", "dbpro-nb"];

/// Hard cap for workspace documents opened into the editor (#377). Reading
/// files is moved off the UI thread, but a multi-hundred-MB buffer would still
/// stall rope insertion and repaint, so oversized files are refused.
pub(super) const WORKSPACE_FILE_MAX_BYTES: u64 = 64 * 1024 * 1024;

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

/// Case-insensitive directory order without allocating a lowercased `String`
/// per entry on every scan (#393).
fn cmp_os_str_ci(a: &OsStr, b: &OsStr) -> std::cmp::Ordering {
    a.to_string_lossy()
        .chars()
        .flat_map(char::to_lowercase)
        .cmp(b.to_string_lossy().chars().flat_map(char::to_lowercase))
        .then_with(|| a.cmp(b))
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
    collected.sort_by(|a, b| {
        let a_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        b_dir
            .cmp(&a_dir)
            .then_with(|| cmp_os_str_ci(&a.file_name(), &b.file_name()))
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
        let file_type = entry
            .file_type()
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        // `file_type` does not follow links, so skip symlinks explicitly — a
        // symlinked dir would otherwise recurse into link cycles (#393).
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
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

/// Diagnostics over an already-built index; shared by the sync path and the
/// background scan worker so file reads stay off the draw path (#393).
pub(super) fn index_diagnostics(
    root_id: &str,
    root_path: &Path,
    index: &[WorkspaceIndexEntry],
) -> Vec<WorkspaceDiagnostic> {
    let mut diagnostics = Vec::new();
    for entry in index.iter().filter(|entry| entry.is_sql) {
        let path = root_path.join(&entry.relative_path);
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("SELECT *") {
                diagnostics.push(WorkspaceDiagnostic {
                    root_id: root_id.to_owned(),
                    relative_path: entry.relative_path.clone(),
                    line: line_idx + 1,
                    message: "avoid SELECT * in workspace SQL".to_owned(),
                    severity: WorkspaceDiagnosticSeverity::Warning,
                });
            }
            if trimmed.contains("DROP TABLE") && !trimmed.contains("IF EXISTS") {
                diagnostics.push(WorkspaceDiagnostic {
                    root_id: root_id.to_owned(),
                    relative_path: entry.relative_path.clone(),
                    line: line_idx + 1,
                    message: "DROP TABLE without IF EXISTS".to_owned(),
                    severity: WorkspaceDiagnosticSeverity::Error,
                });
            }
        }
    }
    diagnostics
}

// ---------------------------------------------------------------------------
// Off-UI-thread workspace I/O (#377, #393)
//
// `scan_directory` and `read_to_string` block; views must never run them on the
// draw path. The operations below are spawned on a dedicated std::thread and
// report back through a channel the state polls per frame — same shape as
// `git_workspace::spawn_git_task` (#391).
// ---------------------------------------------------------------------------

/// One root scanned on a worker thread, ready to be applied to UI state.
#[derive(Debug)]
pub(super) struct ScannedRoot {
    pub(super) id: String,
    pub(super) path: PathBuf,
    pub(super) result: Result<(Vec<WorkspaceFileNode>, Vec<WorkspaceIndexEntry>), String>,
    pub(super) diagnostics: Vec<WorkspaceDiagnostic>,
}

fn scan_root(id: String, path: PathBuf) -> ScannedRoot {
    let result = (|| {
        if !path.is_dir() {
            return Err(format!("not a folder: {}", path.display()));
        }
        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
        let mut count = 0usize;
        let tree = scan_directory(&canonical, &canonical, 0, &mut count)?;
        let index = flatten_index(&id, &tree);
        Ok::<_, String>((canonical, tree, index))
    })();
    match result {
        Ok((canonical, tree, index)) => {
            let diagnostics = index_diagnostics(&id, &canonical, &index);
            ScannedRoot {
                id,
                path: canonical,
                result: Ok((tree, index)),
                diagnostics,
            }
        }
        Err(error) => ScannedRoot {
            id,
            path,
            result: Err(error),
            diagnostics: Vec::new(),
        },
    }
}

/// Scan `(id, path)` roots on a worker thread; the outcome arrives on the
/// returned receiver so the egui frame never blocks on `read_dir` (#393).
pub(super) fn spawn_scan_task(roots: Vec<(String, PathBuf)>) -> std::sync::mpsc::Receiver<Vec<ScannedRoot>> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let scanned = roots
            .into_iter()
            .map(|(id, path)| scan_root(id, path))
            .collect();
        let _ = sender.send(scanned);
    });
    receiver
}

/// What a finished file read should do once the content arrives.
#[derive(Debug)]
pub(super) enum FileReadOutcome {
    /// Open the file as a new document (or switch to it if already open).
    Open {
        path: PathBuf,
        result: Result<(String, Option<u64>), String>,
    },
}

/// Read `path` on a worker thread with a size cap; the outcome arrives on the
/// returned receiver so the egui frame never blocks on disk I/O (#377).
pub(super) fn spawn_file_read_task(path: PathBuf) -> std::sync::mpsc::Receiver<FileReadOutcome> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let metadata = std::fs::metadata(&path).map_err(|error| format!("{}: {error}", path.display()))?;
            if metadata.len() > WORKSPACE_FILE_MAX_BYTES {
                return Err(format!(
                    "{} is {} MB — too large to open in the editor ({} MB limit)",
                    path.display(),
                    metadata.len() / (1024 * 1024),
                    WORKSPACE_FILE_MAX_BYTES / (1024 * 1024)
                ));
            }
            let content = std::fs::read_to_string(&path).map_err(|error| {
                if error.kind() == std::io::ErrorKind::InvalidData {
                    format!("{} is not valid UTF-8 — convert its encoding first", path.display())
                } else {
                    format!("{}: {error}", path.display())
                }
            })?;
            let mtime = metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs());
            Ok((content, mtime))
        })();
        let _ = sender.send(FileReadOutcome::Open { path, result });
    });
    receiver
}
