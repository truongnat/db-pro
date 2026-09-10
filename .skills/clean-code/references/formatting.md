# Formatting & Layout — Định dạng mã nguồn

Định dạng là **giao tiếp**. Repo đã cố định công cụ; đừng tranh luận style bằng tay — chạy formatter
và tập trung vào phần formatter không làm được: **bố cục theo ý nghĩa**.

## Công cụ đã cố định trong repo

| Ngôn ngữ | Công cụ | Cấu hình | Lệnh kiểm tra |
|----------|---------|----------|----------------|
| TS/TSX/CSS/MD | Prettier | `frontend/.prettierrc`: `printWidth: 100`, `semi: true`, double quote, `trailingComma: all` | `cd frontend && pnpm run format:check` |
| TS/TSX | ESLint | `frontend/eslint.config.js` (typescript-eslint recommended, `no-unused-vars` với `_` prefix) | `pnpm run lint` |
| Rust | rustfmt | `.rustfmt.toml`: `max_width = 120`, `tab_spaces = 4`, edition 2021 | `cargo fmt --all -- --check` |
| Rust | Clippy | mặc định workspace | `cargo clippy --workspace --all-targets` |

**Không** commit thay đổi format hàng loạt lẫn với thay đổi logic — tách PR / commit riêng để diff review được.

## Giới hạn kích thước

| Đơn vị | Mục tiêu | Ngưỡng cảnh báo | Ngưỡng chặn |
|--------|----------|-----------------|-------------|
| Dòng (TS) | ≤ 100 ký tự (Prettier lo) | — | — |
| Dòng (Rust) | ≤ 120 ký tự (rustfmt lo) | — | — |
| Hàm | ≤ 30 dòng | > 50 | > 100 |
| React component | ≤ 150 dòng | > 200 | > 300 |
| File TS/TSX | ≤ 300 dòng | > 400 | > 600 |
| File Rust | ≤ 500 dòng | > 800 | > 1200 |
| Độ sâu lồng | ≤ 2 | 3 | > 3 |
| Tham số hàm | ≤ 2 | 3 | > 3 |

Ngưỡng là **tín hiệu để nhìn kỹ**, không phải luật tuyệt đối. Một `match` 80 nhánh ánh xạ SQLSTATE có thể hợp lý;
một hàm 80 dòng trộn IO + business logic thì không.

## Định dạng dọc (Vertical Formatting)

### 1. Ẩn dụ tờ báo (Newspaper metaphor)

Đọc từ trên xuống: tiêu đề (tên file/module) → tóm tắt (public API, kiểu chính) → chi tiết (helper private).

```text
imports
types / interfaces / consts
public function (điểm vào)
  ↓ hàm nó gọi
    ↓ hàm cấp thấp hơn
tests (Rust: #[cfg(test)] mod tests ở cuối)
```

### 2. Khoảng trắng dọc tách khái niệm

```ts
// ✗ Sai — mọi thứ dính chùm
const tabs = useWorkspaceStore((s) => s.tabs);
const activeId = useWorkspaceStore((s) => s.activeTabId);
const close = useWorkspaceStore((s) => s.closeTab);
const [confirming, setConfirming] = useState<string | null>(null);
const handleClose = useCallback((id: string) => { ... }, [close]);
useEffect(() => { ... }, [activeId]);

// ✓ Đúng — mỗi nhóm một "đoạn văn"
const tabs = useWorkspaceStore((s) => s.tabs);
const activeId = useWorkspaceStore((s) => s.activeTabId);
const close = useWorkspaceStore((s) => s.closeTab);

const [confirming, setConfirming] = useState<string | null>(null);

const handleClose = useCallback((id: string) => { ... }, [close]);

useEffect(() => { ... }, [activeId]);
```

Không quá **1 dòng trống** liên tiếp (Prettier ép).

### 3. Mật độ dọc — thứ liên quan ở gần nhau

- Khai báo biến gần chỗ dùng đầu tiên, không dồn hết lên đầu hàm.
- Biến điều khiển vòng lặp khai báo trong lệnh lặp.
- Hàm gọi và hàm được gọi ở gần nhau, caller **ở trên** callee.
- Các hàm có "họ hàng khái niệm" (`quotePg`, `quoteSqlite`) đặt liền nhau.

### 4. Thứ tự import

Prettier không sắp import; giữ thủ công (hoặc bật plugin):

```ts
// 1. Node/built-in & framework
import { useEffect, useMemo } from "react";
// 2. Thư viện bên thứ ba
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
// 3. Alias nội bộ (@/...)
import { Button } from "@/components/ui/button";
import type { Connection } from "@/commons/types";
// 4. Tương đối
import { buildTree } from "./build-tree";
```

Rust: `std` → external crates → `db_pro_*` → `crate::` / `super::` (rustfmt `group_imports` có thể ép, hiện chưa bật).

### 5. Thứ tự trong React component

```tsx
export function Component(props: Props) {
  // 1. hooks lấy dữ liệu / store
  // 2. state cục bộ
  // 3. derived values (useMemo)
  // 4. callbacks (useCallback / handlers)
  // 5. effects
  // 6. early returns (loading / error / empty)
  // 7. return JSX
}
```

### 6. Thứ tự trong file Rust

```rust
// 1. use
// 2. const / static
// 3. pub types (struct/enum/trait)
// 4. impl blocks (pub trước, private sau)
// 5. free functions (pub trước, private sau)
// 6. #[cfg(test)] mod tests
```

## Định dạng ngang (Horizontal Formatting)

- Dòng > 100/120 ký tự: formatter tự bẻ, nhưng nếu **thường xuyên** bị bẻ → biểu thức quá phức tạp, trích biến/hàm.
- Không canh cột bằng khoảng trắng (`name    = 1`) — formatter phá, và nó khuyến khích danh sách khai báo dài.
- Điều kiện dài trong `if` → đặt tên:

```ts
// ✗ Sai
if (tab.kind === "query" && tab.isDirty && !tab.isPinned && tabs.filter((t) => t.kind === "query").length > 1) { ... }

// ✓ Đúng
const isClosableDirtyQuery = tab.kind === "query" && tab.isDirty && !tab.isPinned;
const hasOtherQueryTabs = tabs.some((t) => t.kind === "query" && t.id !== tab.id);
if (isClosableDirtyQuery && hasOtherQueryTabs) { ... }
```

## Tách file khi nào

Tín hiệu:

- File cần "banner comment" để điều hướng.
- Cuộn > 2 màn hình để tìm hàm.
- File có > 1 lý do thay đổi (ví dụ `schema.ts` chứa cả type, fetcher, formatter, và component).
- Nhiều người thường xuyên conflict trên cùng file.

Cách tách theo cấu trúc repo hiện có:

```text
frontend/src/modules/<feature>/
  components/    UI thuần, nhận props
  hooks/         gắn store / query
  services/      gọi Tauri command, không phụ thuộc React
  utils/         hàm thuần, unit test dễ
  types.ts
  __tests__/
```

```text
crates/<crate>/src/<area>/
  mod.rs         re-export public API
  <concern>.rs   mỗi concern một file
```

## Quy ước team (không có formatter ép)

- Một câu lệnh mỗi dòng; không `if (x) return;` dính với lệnh khác trên cùng dòng khi có nhiều nhánh.
- Luôn dùng `{}` cho `if`/`for` nhiều dòng (Prettier giữ nguyên `if (x) return;` một dòng — chấp nhận).
- Tên file: `kebab-case` cho TS/TSX (theo repo hiện tại), `snake_case.rs` cho Rust.
- Test đặt cạnh code (`__tests__/` hoặc `*.test.ts`), tên test mô tả hành vi.

## Checklist nhanh

- [ ] `pnpm run format:check` / `cargo fmt --check` pass
- [ ] File / hàm / component trong ngưỡng bảng trên
- [ ] Đọc file từ trên xuống hiểu được mà không nhảy lung tung
- [ ] Khoảng trắng dọc tách các khái niệm
- [ ] Biểu thức điều kiện dài đã được đặt tên
- [ ] Không trộn commit format với commit logic
