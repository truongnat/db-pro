// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
// allow: IDE workspace operations remain a persisted contract while shell adoption is incremental.
#![allow(dead_code)]

use super::ide_workspace_scan::{flatten_index, index_diagnostics, scan_directory, ScannedRoot};
pub(crate) use super::ide_workspace_types::*;

use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

impl IdeWorkspaceState {
    pub fn primary_root(&self) -> Option<&WorkspaceRoot> {
        self.roots.get(self.active_root)
    }

    pub fn primary_path(&self) -> Option<&Path> {
        self.primary_root().map(|root| root.path.as_path())
    }

    /// Back-compat for callers that still look at a single `root`.
    pub fn root(&self) -> Option<&PathBuf> {
        self.primary_root().map(|root| &root.path)
    }

    pub fn tree(&self) -> &[WorkspaceFileNode] {
        self.primary_root().map(|root| root.tree.as_slice()).unwrap_or(&[])
    }

    pub fn index(&self) -> Vec<WorkspaceIndexEntry> {
        self.roots.iter().flat_map(|root| root.index.clone()).collect()
    }

    pub fn open_root(&mut self, root: PathBuf) -> Result<(), String> {
        self.roots.clear();
        self.active_root = 0;
        self.add_root(root)
    }

    pub fn add_root(&mut self, root: PathBuf) -> Result<(), String> {
        if !root.is_dir() {
            return Err(format!("not a folder: {}", root.display()));
        }
        let canonical = root.canonicalize().unwrap_or(root);
        if self.roots.iter().any(|existing| existing.path == canonical) {
            return Ok(());
        }
        let id = format!("root-{}", self.roots.len() + 1);
        let mut count = 0usize;
        let tree = scan_directory(&canonical, &canonical, 0, &mut count)?;
        let index = flatten_index(&id, &tree);
        self.roots.push(WorkspaceRoot {
            id,
            path: canonical.clone(),
            tree,
            index,
        });
        self.active_root = self.roots.len() - 1;
        self.last_error = None;
        self.remember_recent(canonical);
        self.seed_default_environments();
        Ok(())
    }

    fn seed_default_environments(&mut self) {
        if self.environments.is_empty() {
            self.environments.push(WorkspaceEnvironment {
                name: "Development".to_owned(),
                connection_id: None,
                database: None,
                schema: None,
            });
            self.environments.push(WorkspaceEnvironment {
                name: "Staging".to_owned(),
                connection_id: None,
                database: None,
                schema: None,
            });
            self.environments.push(WorkspaceEnvironment {
                name: "Production".to_owned(),
                connection_id: None,
                database: None,
                schema: None,
            });
            self.active_environment = 0;
        }
    }

    /// Apply roots scanned on a worker thread (`spawn_scan_task`, #393).
    /// `replace` clears existing roots/diagnostics first; otherwise new roots
    /// are appended and deduped by canonical path like `add_root`.
    /// Returns `(indexed file count, first scan error if any)`.
    pub fn apply_scanned_roots(
        &mut self,
        scanned: Vec<ScannedRoot>,
        replace: bool,
    ) -> (usize, Option<String>) {
        if replace {
            self.roots.clear();
            self.workspace_diagnostics.clear();
            self.active_root = 0;
        }
        let mut error = None;
        for root in scanned {
            let ScannedRoot {
                id,
                path,
                result,
                diagnostics,
            } = root;
            match result {
                Ok((tree, index)) => {
                    if self.roots.iter().any(|existing| existing.path == path) {
                        continue;
                    }
                    self.roots.push(WorkspaceRoot {
                        id,
                        path: path.clone(),
                        tree,
                        index,
                    });
                    self.active_root = self.roots.len() - 1;
                    self.workspace_diagnostics.extend(diagnostics);
                    self.remember_recent(path);
                }
                Err(err) => {
                    if error.is_none() {
                        error = Some(err);
                    }
                }
            }
        }
        self.seed_default_environments();
        if !self.roots.is_empty() {
            self.last_error = None;
        }
        let indexed = self.roots.iter().map(|root| root.index.len()).sum();
        (indexed, error)
    }

    pub fn remove_active_root(&mut self) {
        if self.roots.is_empty() {
            return;
        }
        self.roots.remove(self.active_root);
        if self.active_root >= self.roots.len() {
            self.active_root = self.roots.len().saturating_sub(1);
        }
        if self.roots.is_empty() {
            self.close();
        }
    }

    pub fn close(&mut self) {
        self.roots.clear();
        self.active_root = 0;
        self.expanded.clear();
        self.last_error = None;
        self.trust = WorkspaceTrust::Untrusted;
        self.workspace_diagnostics.clear();
        self.last_task = None;
        self.schema_fingerprint = None;
        self.schema_drift_message = None;
    }

    pub fn set_trusted(&mut self, trusted: bool) {
        self.trust = if trusted {
            WorkspaceTrust::Trusted
        } else {
            WorkspaceTrust::Untrusted
        };
    }

    pub fn is_trusted(&self) -> bool {
        matches!(self.trust, WorkspaceTrust::Trusted)
    }

    pub fn refresh(&mut self) -> Result<(), String> {
        let roots = self.roots.clone();
        self.roots.clear();
        for root in roots {
            self.add_root(root.path)?;
        }
        Ok(())
    }

    pub fn root_label(&self) -> String {
        self.primary_root()
            .and_then(|root| root.path.file_name().map(|name| name.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "No folder".to_owned())
    }

    pub fn absolute_for_relative(&self, relative: &str) -> Option<PathBuf> {
        let (root, relative_path) = self.root_and_relative_path(relative)?;
        let candidate = root.path.join(relative_path);
        existing_ancestor_within_root(&root.path, &candidate).then_some(candidate)
    }

    fn root_and_relative_path<'a>(&'a self, relative: &'a str) -> Option<(&'a WorkspaceRoot, &'a Path)> {
        let (root, relative) = if let Some((root_id, path)) = relative.split_once("::") {
            (self.roots.iter().find(|root| root.id == root_id)?, path)
        } else {
            (self.primary_root()?, relative)
        };
        let relative_path = Path::new(relative);
        safe_relative_path(relative_path).then_some((root, relative_path))
    }

    pub fn qualified_relative(root_id: &str, relative: &str) -> String {
        format!("{root_id}::{relative}")
    }

    fn remember_recent(&mut self, root: PathBuf) {
        self.recent_roots.retain(|item| item != &root);
        self.recent_roots.insert(0, root);
        if self.recent_roots.len() > WORKSPACE_RECENT_MAX {
            self.recent_roots.truncate(WORKSPACE_RECENT_MAX);
        }
    }

    pub fn create_file(&mut self, relative_dir: &str, name: &str, contents: &str) -> Result<PathBuf, String> {
        if !safe_name(name) {
            return Err("file name must be one path component".to_owned());
        }
        let root_id = self
            .primary_root()
            .map(|root| root.id.clone())
            .ok_or_else(|| "no workspace root".to_owned())?;
        let directory = self
            .absolute_for_relative(&Self::qualified_relative(&root_id, relative_dir))
            .ok_or_else(|| "directory escapes workspace root".to_owned())?;
        std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        if self
            .absolute_for_relative(&Self::qualified_relative(&root_id, relative_dir))
            .is_none()
        {
            return Err("directory escapes workspace root".to_owned());
        }
        let path = directory.join(name);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| error.to_string())?;
        file.write_all(contents.as_bytes()).map_err(|e| e.to_string())?;
        self.refresh()?;
        Ok(path)
    }

    pub fn create_folder(&mut self, relative_dir: &str, name: &str) -> Result<PathBuf, String> {
        if !safe_name(name) {
            return Err("folder name must be one path component".to_owned());
        }
        let (root_id, root_path) = self
            .primary_root()
            .map(|root| (root.id.clone(), root.path.clone()))
            .ok_or_else(|| "no workspace root".to_owned())?;
        let directory = self
            .absolute_for_relative(&Self::qualified_relative(&root_id, relative_dir))
            .ok_or_else(|| "directory escapes workspace root".to_owned())?;
        std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let path = directory.join(name);
        if !existing_ancestor_within_root(&root_path, &path) {
            return Err("folder escapes workspace root".to_owned());
        }
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        self.refresh()?;
        Ok(path)
    }

    pub fn rename_path(&mut self, relative: &str, new_name: &str) -> Result<PathBuf, String> {
        if !safe_name(new_name) {
            return Err("new name must be one path component".to_owned());
        }
        let (root, relative_path) = self
            .root_and_relative_path(relative)
            .ok_or_else(|| "path escapes workspace root or is missing".to_owned())?;
        if relative_path_is_root(relative_path) {
            return Err("cannot rename workspace root".to_owned());
        }
        let root_id = root.id.clone();
        let root_path = root.path.clone();
        let absolute = self
            .absolute_for_relative(relative)
            .ok_or_else(|| "path escapes workspace root or is missing".to_owned())?;
        let parent = absolute.parent().ok_or_else(|| "no parent".to_owned())?;
        let dest = parent.join(new_name);
        let relative_dest = dest
            .strip_prefix(&root_path)
            .map_err(|_| "path escapes workspace root")?;
        let qualified_dest = Self::qualified_relative(&root_id, &relative_dest.to_string_lossy());
        if self.absolute_for_relative(&qualified_dest).is_none() {
            return Err("new path escapes workspace root".to_owned());
        }
        std::fs::rename(&absolute, &dest).map_err(|e| e.to_string())?;
        self.refresh()?;
        Ok(dest)
    }

    pub fn delete_path(&mut self, relative: &str) -> Result<(), String> {
        let (root, relative_path) = self
            .root_and_relative_path(relative)
            .ok_or_else(|| "path escapes workspace root".to_owned())?;
        if relative_path_is_root(relative_path) {
            return Err("cannot delete workspace root".to_owned());
        }
        #[cfg(unix)]
        remove_workspace_path_no_follow(&root.path, relative_path).map_err(|e| e.to_string())?;
        #[cfg(not(unix))]
        {
            let absolute = root.path.join(relative_path);
            let parent = absolute.parent().ok_or_else(|| "no parent".to_owned())?;
            if !existing_ancestor_within_root(&root.path, parent) {
                return Err("parent path escapes workspace root".to_owned());
            }
            let metadata = std::fs::symlink_metadata(&absolute).map_err(|e| e.to_string())?;
            if metadata.file_type().is_symlink() {
                std::fs::remove_file(&absolute).map_err(|e| e.to_string())?;
            } else if metadata.is_dir() {
                std::fs::remove_dir_all(&absolute).map_err(|e| e.to_string())?;
            } else {
                std::fs::remove_file(&absolute).map_err(|e| e.to_string())?;
            }
        }
        self.refresh()?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let mut hits = Vec::new();
        if query.is_empty() {
            return hits;
        }
        let needle = query.to_lowercase();
        for root in &self.roots {
            for entry in &root.index {
                if hits.len() >= limit {
                    return hits;
                }
                let path = root.path.join(&entry.relative_path);
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                for (line_idx, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&needle) {
                        hits.push(SearchHit {
                            root_id: root.id.clone(),
                            relative_path: entry.relative_path.clone(),
                            line: line_idx + 1,
                            preview: line.chars().take(160).collect(),
                        });
                        if hits.len() >= limit {
                            return hits;
                        }
                    }
                }
            }
        }
        hits
    }

    pub fn preview_replace(&self, query: &str, replacement: &str) -> Vec<ReplacePreview> {
        if query.is_empty() {
            return Vec::new();
        }
        let mut previews = Vec::new();
        for root in &self.roots {
            for entry in &root.index {
                let path = root.path.join(&entry.relative_path);
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let count = content.matches(query).count();
                if count > 0 {
                    let _ = replacement; // preview only
                    previews.push(ReplacePreview {
                        root_id: root.id.clone(),
                        relative_path: entry.relative_path.clone(),
                        replacements: count,
                    });
                }
            }
        }
        previews
    }

    pub fn apply_replace(&mut self, query: &str, replacement: &str) -> Result<usize, String> {
        if !self.is_trusted() {
            return Err("trust the workspace before replace-in-files".to_owned());
        }
        if query.is_empty() {
            return Err("empty search".to_owned());
        }
        let mut total = 0usize;
        for root in &self.roots {
            for entry in &root.index {
                let path = root.path.join(&entry.relative_path);
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                if !content.contains(query) {
                    continue;
                }
                let updated = content.replace(query, replacement);
                let count = content.matches(query).count();
                std::fs::write(&path, updated).map_err(|e| e.to_string())?;
                total += count;
            }
        }
        self.refresh()?;
        Ok(total)
    }

    pub fn find_references(&self, symbol: &str, limit: usize) -> Vec<ReferenceHit> {
        self.search(symbol, limit)
            .into_iter()
            .map(|hit| ReferenceHit {
                root_id: hit.root_id,
                relative_path: hit.relative_path,
                line: hit.line,
                preview: hit.preview,
            })
            .collect()
    }

    pub fn scan_diagnostics(&mut self) {
        let mut diagnostics = Vec::new();
        for root in &self.roots {
            diagnostics.extend(index_diagnostics(&root.id, &root.path, &root.index));
        }
        self.workspace_diagnostics = diagnostics;
    }

    pub fn detect_migrations(&self) -> Vec<MigrationEntry> {
        let mut entries = Vec::new();
        for root in &self.roots {
            for entry in root.index.iter().filter(|entry| entry.is_sql) {
                let lower = entry.relative_path.to_lowercase();
                if !(lower.contains("migration") || lower.contains("/migrate") || lower.starts_with("migrations/")) {
                    continue;
                }
                let version = Path::new(&entry.relative_path)
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
                    .unwrap_or_else(|| entry.relative_path.clone());
                entries.push(MigrationEntry {
                    relative_path: entry.relative_path.clone(),
                    absolute_path: root.path.join(&entry.relative_path),
                    version,
                    status: MigrationStatus::Unknown,
                });
            }
        }
        entries.sort_by(|a, b| a.version.cmp(&b.version));
        entries
    }

    pub fn run_task(&mut self, command_line: &str) -> Result<TaskRunResult, String> {
        if !self.is_trusted() {
            return Err("trust the workspace before running tasks/terminal commands".to_owned());
        }
        let cwd = self
            .primary_path()
            .ok_or_else(|| "open a workspace folder first".to_owned())?
            .to_path_buf();
        let started = Instant::now();
        #[cfg(target_os = "windows")]
        let output = Command::new("cmd")
            .args(["/C", command_line])
            .current_dir(&cwd)
            .output();
        #[cfg(not(target_os = "windows"))]
        let output = Command::new("sh")
            .args(["-lc", command_line])
            .current_dir(&cwd)
            .output();
        let output = output.map_err(|e| e.to_string())?;
        let result = TaskRunResult {
            command: command_line.to_owned(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration_ms: started.elapsed().as_millis() as u64,
        };
        self.last_task = Some(result.clone());
        Ok(result)
    }

    pub fn export_schema_snapshot(&self, schema_sql: &str) -> Result<PathBuf, String> {
        let root = self
            .primary_path()
            .ok_or_else(|| "open a workspace folder first".to_owned())?;
        let dir = root.join(".dbpro");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("schema-snapshot.sql");
        std::fs::write(&path, schema_sql).map_err(|e| e.to_string())?;
        Ok(path)
    }

    pub fn rename_symbol_across_sql(&mut self, from: &str, to: &str) -> Result<usize, String> {
        if from.is_empty() || to.is_empty() {
            return Err("from/to required".to_owned());
        }
        if !self.is_trusted() {
            return Err("trust the workspace before refactoring".to_owned());
        }
        self.apply_replace(from, to)
    }

    pub fn active_environment(&self) -> Option<&WorkspaceEnvironment> {
        self.environments.get(self.active_environment)
    }

    pub fn set_active_environment(&mut self, index: usize) {
        if index < self.environments.len() {
            self.active_environment = index;
        }
    }

    pub fn update_schema_fingerprint(&mut self, fingerprint: u64) {
        if let Some(previous) = self.schema_fingerprint {
            if previous != fingerprint {
                self.schema_drift_message = Some(format!(
                    "Schema drift detected (fingerprint {previous} → {fingerprint})"
                ));
            }
        }
        self.schema_fingerprint = Some(fingerprint);
        self.last_drift_check = Some(Instant::now());
    }

    pub fn dependency_edges(&self) -> Vec<DependencyEdge> {
        let mut edges = Vec::new();
        for root in &self.roots {
            for entry in root.index.iter().filter(|entry| entry.is_sql) {
                let path = root.path.join(&entry.relative_path);
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                for token in extract_table_like_tokens(&content) {
                    edges.push(DependencyEdge {
                        from_file: format!("{}::{}", root.id, entry.relative_path),
                        object_name: token,
                    });
                }
            }
        }
        edges
    }

    pub fn create_sandbox_sqlite_copy(&self, source_db: &Path) -> Result<PathBuf, String> {
        if !self.is_trusted() {
            return Err("trust the workspace before creating a sandbox".to_owned());
        }
        let root = self
            .primary_path()
            .ok_or_else(|| "open a workspace folder first".to_owned())?;
        let sandbox_dir = root.join(".dbpro").join("sandbox");
        std::fs::create_dir_all(&sandbox_dir).map_err(|e| e.to_string())?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let dest = sandbox_dir.join(format!("sandbox-{stamp}.sqlite"));
        std::fs::copy(source_db, &dest).map_err(|e| e.to_string())?;
        Ok(dest)
    }

    pub fn load_notebook(path: &Path) -> Result<Vec<NotebookCell>, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let doc: NotebookDocument = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        Ok(doc
            .cells
            .into_iter()
            .map(|cell| NotebookCell {
                kind: if cell.kind.eq_ignore_ascii_case("sql") {
                    NotebookCellKind::Sql
                } else {
                    NotebookCellKind::Markdown
                },
                source: cell.source,
            })
            .collect())
    }

    pub fn save_notebook(path: &Path, cells: &[NotebookCell]) -> Result<(), String> {
        let doc = NotebookDocument {
            version: 1,
            cells: cells
                .iter()
                .map(|cell| NotebookCellSerde {
                    kind: match cell.kind {
                        NotebookCellKind::Markdown => "markdown".to_owned(),
                        NotebookCellKind::Sql => "sql".to_owned(),
                    },
                    source: cell.source.clone(),
                })
                .collect(),
        };
        let raw = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, raw).map_err(|e| e.to_string())
    }

    pub fn parse_sql_tests(content: &str) -> Vec<(String, String)> {
        // Minimal convention: -- test: name\nSQL until next test marker.
        let mut tests = Vec::new();
        let mut current_name: Option<String> = None;
        let mut current_sql = String::new();
        for line in content.lines() {
            if let Some(rest) = line.trim().strip_prefix("-- test:") {
                if let Some(name) = current_name.take() {
                    tests.push((name, current_sql.trim().to_owned()));
                    current_sql.clear();
                }
                current_name = Some(rest.trim().to_owned());
            } else if current_name.is_some() {
                current_sql.push_str(line);
                current_sql.push('\n');
            }
        }
        if let Some(name) = current_name {
            tests.push((name, current_sql.trim().to_owned()));
        }
        tests
    }

    pub fn apply_workspace_patch(&mut self, relative: &str, new_contents: &str) -> Result<(), String> {
        if !self.is_trusted() {
            return Err("trust the workspace before applying AI patches".to_owned());
        }
        let path = self
            .absolute_for_relative(relative)
            .ok_or_else(|| "missing file".to_owned())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        if self.absolute_for_relative(relative).as_deref() != Some(path.as_path()) {
            return Err("patch path escapes workspace root".to_owned());
        }
        std::fs::write(&path, new_contents).map_err(|e| e.to_string())?;
        self.refresh()?;
        Ok(())
    }
}

fn relative_path_is_root(path: &Path) -> bool {
    path.components().all(|component| matches!(component, Component::CurDir))
}


#[cfg(unix)]
fn remove_workspace_path_no_follow(root: &Path, relative: &Path) -> std::io::Result<()> {
    use rustix::fd::AsFd;
    use rustix::fs::{openat, unlinkat, AtFlags, Dir, Mode, OFlags, CWD};
    use rustix::io::Errno;
    use std::ffi::{CStr, CString};
    use std::os::fd::OwnedFd;
    use std::os::unix::ffi::OsStrExt;

    fn open_dir_at(parent: impl AsFd, name: &CStr) -> std::io::Result<OwnedFd> {
        openat(
            parent,
            name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(std::io::Error::from)
    }

    fn open_root(root: &Path) -> std::io::Result<OwnedFd> {
        let mut current = openat(
            CWD,
            "/",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(std::io::Error::from)?;
        for component in root.components() {
            match component {
                std::path::Component::RootDir => {}
                std::path::Component::Normal(name) => {
                    let name = CString::new(name.as_bytes())
                        .map_err(|_| std::io::Error::from(Errno::INVAL))?;
                    current = open_dir_at(&current, &name)?;
                }
                _ => return Err(std::io::Error::from(Errno::INVAL)),
            }
        }
        Ok(current)
    }

    fn remove_entry(parent: impl AsFd, name: &CStr) -> std::io::Result<()> {
        match open_dir_at(&parent, name) {
            Ok(directory) => {
                let mut entries = Dir::read_from(&directory).map_err(std::io::Error::from)?;
                let names = entries
                    .by_ref()
                    .map(|entry| entry.map(|entry| entry.file_name().to_owned()).map_err(std::io::Error::from))
                    .collect::<std::io::Result<Vec<_>>>()?;
                for child in names {
                    if child.to_bytes() != b"." && child.to_bytes() != b".." {
                        remove_entry(&directory, &child)?;
                    }
                }
                unlinkat(&parent, name, AtFlags::REMOVEDIR).map_err(std::io::Error::from)
            }
            Err(_) => unlinkat(&parent, name, AtFlags::empty()).map_err(std::io::Error::from),
        }
    }

    let mut components = relative.components();
    let name = match components.next() {
        Some(std::path::Component::Normal(name)) if components.next().is_none() => name,
        Some(std::path::Component::Normal(_)) => {
            let mut all = relative.components().collect::<Vec<_>>();
            let name = all.pop().ok_or_else(|| std::io::Error::from(Errno::INVAL))?;
            let std::path::Component::Normal(name) = name else {
                return Err(std::io::Error::from(Errno::INVAL));
            };
            let mut parent = open_root(root)?;
            for component in all {
                let std::path::Component::Normal(part) = component else {
                    return Err(std::io::Error::from(Errno::INVAL));
                };
                let part = CString::new(part.as_bytes())
                    .map_err(|_| std::io::Error::from(Errno::INVAL))?;
                parent = open_dir_at(&parent, &part)?;
            }
            let name = CString::new(name.as_bytes())
                .map_err(|_| std::io::Error::from(Errno::INVAL))?;
            return remove_entry(&parent, &name);
        }
        _ => return Err(std::io::Error::from(Errno::INVAL)),
    };
    let parent = open_root(root)?;
    let name = CString::new(name.as_bytes()).map_err(|_| std::io::Error::from(Errno::INVAL))?;
    remove_entry(&parent, &name)
}

fn safe_relative_path(path: &Path) -> bool {
    !path.is_absolute()
        && path.components().all(|component| {
            matches!(component, Component::Normal(_) | Component::CurDir)
        })
}

fn safe_name(name: &str) -> bool {
    let mut components = Path::new(name).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn existing_ancestor_within_root(root: &Path, candidate: &Path) -> bool {
    let Ok(canonical_root) = root.canonicalize() else {
        return false;
    };
    let mut ancestor = candidate;
    loop {
        match std::fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let Some(parent) = ancestor.parent() else {
                    return false;
                };
                ancestor = parent;
            }
            Err(_) => return false,
        }
    }
    ancestor
        .canonicalize()
        .is_ok_and(|canonical_ancestor| canonical_ancestor.starts_with(canonical_root))
}

fn extract_table_like_tokens(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let upper = sql.to_uppercase();
    for keyword in [" FROM ", " JOIN ", " UPDATE ", " INTO ", " TABLE "] {
        let mut rest = upper.as_str();
        while let Some(idx) = rest.find(keyword) {
            let after = &rest[idx + keyword.len()..];
            let token: String = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
                .collect();
            if !token.is_empty() {
                out.push(token.to_lowercase());
            }
            rest = &after[token.len().min(after.len())..];
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Back-compat helper used by older call sites / tests.
#[allow(dead_code)]
pub fn search_workspace_files(
    root: &Path,
    index: &[WorkspaceIndexEntry],
    query: &str,
    limit: usize,
) -> Vec<(String, usize, String)> {
    let mut state = IdeWorkspaceState::default();
    let _ = state.open_root(root.to_path_buf());
    let _ = index;
    state
        .search(query, limit)
        .into_iter()
        .map(|hit| (hit.relative_path, hit.line, hit.preview))
        .collect()
}

pub fn fingerprint_schema_names(names: &[String]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    for name in names {
        name.hash(&mut hasher);
    }
    hasher.finish()
}

pub fn measure_local_benchmark(cases: &[BenchmarkCase], iterations: usize) -> Vec<BenchmarkResult> {
    cases
        .iter()
        .map(|case| {
            let mut runs = Vec::new();
            for _ in 0..iterations.max(1) {
                let started = Instant::now();
                // Local CPU stand-in: parse length / hash — real DB timing stays on QueryService.
                let _ = case.sql.len().saturating_mul(17);
                std::thread::sleep(Duration::from_millis(0));
                runs.push(started.elapsed().as_millis() as u64);
            }
            let avg = if runs.is_empty() {
                0
            } else {
                runs.iter().sum::<u64>() / runs.len() as u64
            };
            BenchmarkResult {
                name: case.name.clone(),
                runs,
                avg_ms: avg,
                error: None,
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/ide_workspace_tests.rs"]
mod tests;

