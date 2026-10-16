# #513 — Workspace rename remaps expanded state

## Goal
`rename_path` trên dir đang mở không làm collapse file tree: `expanded` phải remap prefix cũ → mới, refresh sau đó vẫn hiển thị node mới ở trạng thái mở.

## Scope
- `crates/app/src/files/ide_workspace.rs::rename_path` — remap `expanded` sau `fs::rename` thành công.
- `delete_path` — prune `expanded` entries của subtree bị xóa (stale entries cùng class, tránh rò rỉ phantom).
- Không đổi key scheme (`expanded` vẫn là plain `relative_path` như `scan_directory` sinh ra); không chuyển sang inode/guid (đề xuất trong issue là alternative, ngoài scope P2 này).

## Non-goals
- Multi-root qualified keys (`root-id::path` trong `expanded`) — hiện `expanded` là plain, tree view chỉ render primary root.
- Async/background scan remap — `rename_path` là sync path duy nhất đổi tên trên disk.

## Architecture
- `WorkspaceFileNode.relative_path`: plain, `/`-separated (`scan_directory`).
- `expanded: BTreeSet<String>`: plain keys, toggle qua `ToggleDirectory(relative_path)`.
- `rename_path(relative, new_name)`: chấp nhận qualified hoặc plain qua `root_and_relative_path`; tính `old_key` (plain) và `new_key` (plain dest strip prefix root); sau `fs::rename` OK thì remap mọi entry `== old_key` hoặc `starts_with(old_key + "/")` sang `new_key` (+ suffix), giữ boundary `/` để `src` không ăn `src2`.
- Entries qualified `id::suffix` (phòng thủ tương lai): remap phần suffix theo cùng quy tắc, giữ prefix id.

## Acceptance
- Rename `src` → `source` khi `expanded={src, src/sub1}` ⇒ `expanded={source, source/sub1}`, tree sau refresh match.
- Rename nested `src/sub1` → `src/renamed` ⇒ chỉ subtree đó remap, `src` và `src/other` giữ nguyên.
- Rename file không đụng `expanded` của dir cha.
- `src` rename không remap `src2` (boundary `/`).
- Delete dir prune subtree khỏi `expanded`.
