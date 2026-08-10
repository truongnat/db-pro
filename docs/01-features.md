# 06 — DB Client — Feature Plan

---

## 1. Focus: PostgreSQL First

PostgreSQL là database chính. Các connector khác (MySQL, SQL Server) bổ sung sau.

---

## 2. Feature Groups

### 2.1 Connection Management

| # | Feature | Mô tả |
|---|---|---|
| CO01001 | Connection list | Danh sách kết nối đã lưu, CRUD |
| CO03001 | Connection editor | Form kết nối: host, port, database, user, password, SSL mode, SSH tunnel |
| CO03002 | Test connection | Kiểm tra kết nối trước khi lưu |
| CO01002 | Connect / Disconnect | Kết nối và ngắt kết nối |

**Connection config fields:**
- name, host, port (mặc định 5432), database, username, password
- ssl_mode: disable / require / verify-ca / verify-full
- ssh_tunnel: bật/tắt, host, port, user, private_key_path
- password mã hoá qua OS keyring (AES-256-GCM)

---

### 2.2 Schema Browser

| # | Feature | Mô tả |
|---|---|---|
| SC01001 | Schema tree | Tree view: schemas → tables → columns |
| SC01002 | Table details | Click table → hiển thị: columns (name, type, nullable, default), indexes, foreign keys, triggers |
| SC01003 | DDL viewer | Hiển thị CREATE TABLE DDL cho table đã chọn |
| SC01004 | View list | Liệt kê views trong schema |
| SC01005 | Function list | Liệt kê functions trong schema |
| SC01006 | Index list | Liệt kê indexes với columns, uniqueness |
| SC01007 | Row count | Hiển thị số dòng của mỗi table |
| SC01008 | ERD diagram | Sơ đồ quan hệ giữa các table (foreign keys), auto-generate từ introspection |

---

### 2.3 SQL Editor

| # | Feature | Mô tả |
|---|---|---|
| QU01001 | SQL editor | Monaco editor, syntax highlight, line numbers |
| QU01002 | Autocomplete | Gợi ý table names, column names, SQL keywords từ introspection |
| QU01003 | Multi-query | Chạy nhiều câu lệnh cách nhau bởi `;` |
| QU01004 | Execute selected | Chạy chỉ phần text được chọn |
| QU01005 | Execute all | Chạy toàn bộ nội dung editor |
| QU01006 | Explain plan | Chạy `EXPLAIN ANALYZE` cho câu lệnh SELECT, hiển thị cost tree |
| QU01007 | Transaction control | Nút BEGIN / COMMIT / ROLLBACK |
| QU01008 | Query history | Lưu lại các câu đã chạy, searchable, click để chạy lại |
| QU01009 | Saved queries | Lưu query hay dùng, tổ chức theo folder |

---

### 2.4 Data Grid — CRUD (trọng tâm)

| # | Feature | Mô tả |
|---|---|---|
| DG01001 | View table data | SELECT * FROM table, virtualized grid, sortable columns |
| DG01002 | Inline edit cell | Click cell → edit → Enter để UPDATE (WHERE pk = value) |
| DG01003 | Add row | Nút "Add row" → empty row ở cuối → Enter để INSERT |
| DG01004 | Delete row | Chọn row → Delete → xác nhận → DELETE WHERE pk |
| DG01005 | Column sort | Click header để sort ASC/DESC |
| DG01006 | Column filter | Filter trên mỗi column (text, number, date) |
| DG01007 | Column resize | Kéo để thay đổi width |
| DG01008 | Column hide/show | Toggle visibility columns |
| DG01009 | Column reorder | Kéo thả để thay đổi thứ tự |
| DG01010 | Pagination | Page navigation (page size: 25/50/100/200) |
| DG01011 | Loading state | Spinner khi đang load data |
| DG01012 | Error inline | Hiển thị lỗi ngay trên cell khi UPDATE/INSERT/DELETE thất bại |
| DG01013 | Refresh | Nút refresh để reload data từ DB |

**CRUD flow:**

```
Edit cell → User edits value → On Enter → UPDATE table SET column = $1 WHERE pk = $2
  → Nếu thành công → cell hiển thị giá trị mới
  → Nếu thất bại → cell highlight đỏ, hiển thị error message

Add row → User fills values → On Enter → INSERT INTO table (cols) VALUES ($1, $2, ...)
  → Nếu thành công → row xuất hiện trong grid
  → Nếu thất bại → row highlight đỏ, hiển thị error

Delete row → User selects row → Click delete → Confirm dialog → DELETE FROM table WHERE pk = $1
  → Nếu thành công → row bị remove khỏi grid
  → Nếu thất bại → row highlight đỏ, hiển thị error
```

---

### 2.5 Export / Import

| # | Feature | Mô tả |
|---|---|---|
| EX01001 | Export CSV | Streaming export, UTF-8 BOM cho Excel compatibility |
| EX01002 | Export JSON | Streaming export |
| EX01003 | Export Excel | Xuất ra .xlsx |
| EX01004 | Import CSV | Import file CSV vào table |
| EX01005 | Import Excel | Import file .xlsx vào table |

---

### 2.6 Transaction Management

| # | Feature | Mô tả |
|---|---|---|
| TR01001 | BEGIN | Bắt đầu transaction |
| TR01002 | COMMIT | Xác nhận transaction |
| TR01003 | ROLLBACK | Hoàn tác transaction |
| TR01004 | Transaction status | Hiển thị trạng thái transaction hiện tại (active / committed / rolled back) |
| TR01005 | Auto-commit toggle | Chế độ auto-commit cho mỗi câu lệnh riêng lẻ |

---

### 2.7 SSH Tunnel

| # | Feature | Mô tả |
|---|---|---|
| TU01001 | SSH tunnel config | Cấu hình tunnel trong connection editor |
| TU01002 | Auto-connect | Kết nối tunnel tự động khi connect to DB |
| TU01003 | Tunnel status | Hiển thị trạng thái tunnel (connected / disconnected / error) |

---

### 2.8 DDL Editor (Create/Edit/Delete Table)

| # | Feature | Mô tả |
|---|---|---|
| DG02001 | Create table | Editor sinh CREATE TABLE DDL từ form (table name, columns, types, PK, FK, indexes) |
| DG02002 | Edit table | Sửa column definition, thêm/drop column, đổi type |
| DG02003 | Delete table | Xóa table (có confirmation) |
| DG02004 | Create view | Editor sinh CREATE VIEW từ SQL |
| DG02005 | Edit view | Sửa view definition |
| DG02006 | Delete view | Xóa view |
| DG02007 | Create function | Editor sinh CREATE FUNCTION từ form (name, language, body, parameters) |
| DG02008 | Edit function | Sửa function body, parameters, return type |
| DG02009 | Delete function | Xóa function |

---

### 2.9 SQL Editor — Advanced

| # | Feature | Mô tả |
|---|---|---|
| QU02001 | SQL formatter | Auto-format SQL: indent, keyword case, line breaks (giống DBeaver/DataGrip) |
| QU02002 | SQL templates | Code snippets: SELECT, INSERT, UPDATE, DELETE, CREATE TABLE templates |
| QU02003 | Query parameter binding | Bind parameters with `$1`, `$2`... syntax, prompt for values before execution |
| QU02004 | Query result copy with headers | Copy grid data to clipboard including column headers |
| QU02005 | Query result export to SQL | Export selected rows as INSERT/UPDATE/DELETE SQL statements |
| QU02006 | Query cancel/kill | Cancel running query, kill active session |
| QU02007 | Auto-commit indicator | Visual indicator showing auto-commit on/off status |
| QU02008 | Query console tabs | Multiple query tabs, each with own connection context |
| QU02009 | SQL script execution order | Run multiple SQL scripts in defined order |
| QU02010 | Query history with diff | Compare current query with previous versions |
| QU02011 | Visual Query Builder | Drag-and-drop query construction (join tables, set filters visually) |
| QU02012 | Outline viewer | Show SQL query structure (CTEs, subqueries, statements) |
| QU02013 | Problem markers / semantic errors | Underline unresolved objects, keyword-as-identifier, syntax errors before execution |
| QU02014 | CTEs support | Parse and autocomplete Common Table Expressions (WITH clauses) |
| QU02015 | STRAIGHT JOIN / LATERAL JOIN | Recognize and handle advanced join types |
| QU02016 | Multi-line variables in SQL scripts | Support for variables across multiple statements |
| QU02017 | Result set zoom | Zoom in/out on result grid (Alt+0 / Alt+9) |
| QU02018 | Advanced paste | Paste multiple records, ignore quotes option |
| QU02019 | Auto-save in SQL Editor | Auto-save scripts to prevent data loss |
| QU02020 | SQL script organization into folders | Organize SQL scripts into folders and subfolders |
| QU02021 | Import/export SQL scripts | Import and export SQL script files |
| QU02022 | Local history | Save local history of every query edit (undo to any previous state) |
| QU02023 | Run configurations | Save and run predefined script configurations with target schema and pre-run tasks |
| QU02024 | Multiple result sets | Display multiple result sets from multi-statement queries in separate tabs |
| QU02025 | Result set metadata view | Show column type, length, nullable, default on result set header |
| QU02026 | GIS data viewer (PostGIS) | Render PostGIS geometry/geography columns on map |
| QU02027 | Mock data generator | Generate random test data for tables (DBeaver Enterprise feature) |
| QU02028 | SQL Debugger (PostgreSQL) | Step-through debugging for SQL functions and procedures |
| QU02029 | Database health monitoring | Monitor database health: connections, locks, slow queries, replication lag |
| QU02030 | Task scheduler | Schedule SQL scripts and data transfer tasks for later/recurrent execution |
| QU02031 | AI Assistant in SQL editor | Generate SQL from natural language, explain queries, fix errors (future/experimental) |

---

### 2.10 Schema Browser — Advanced

| # | Feature | Mô tả |
|---|---|---|
| SC02001 | Database object search | Quick search (Ctrl+Shift+O) for tables, views, functions, columns across all schemas |
| SC02002 | Index management | Create, edit, delete indexes with UI form |
| SC02003 | Trigger management | Create, edit, delete triggers with SQL editor |
| SC02004 | Sequence management | Create, edit, delete sequences |
| SC02005 | Schema diff | Compare two schemas/tables, generate sync SQL |
| SC02006 | Data diff | Compare two tables' data, generate sync SQL |
| SC02007 | Database object dependency viewer | Show which objects reference a given table/column |
| SC02008 | Database object dependency graph | Visual graph of table relationships beyond ERD |
| SC02009 | Database object rename/refactoring | Rename table/column and propagate to all dependent objects |
| SC02010 | Database structure compare | Compare schema structure between two databases |
| SC02011 | Database object quick navigation | Jump to any object by name quickly |
| SC02012 | Table partition management | View and manage table partitions |
| SC02013 | Tablespace management | View and manage tablespaces |

---

### 2.11 Connection Management — Advanced

| # | Feature | Mô tả |
|---|---|---|
| CO02001 | Connection color coding | Color label per connection for quick visual identification |
| CO02002 | Connection tags/categories | Tag connections (dev, staging, production) for filtering |
| CO02003 | Connection groups | Group connections by project/environment |
| CO02004 | Recent connections quick access | Quick access list of recently used connections |
| CO02005 | Bootstrap SQL | Run SQL queries automatically on connect |
| CO02006 | Connection monitoring | Show active connections, query count, uptime |

---

### 2.12 Data Grid — Advanced

| # | Feature | Mô tả |
|---|---|---|
| DG03001 | WHERE clause filter builder | Visual filter builder for grid data (like DBeaver data filter) |
| DG03002 | Query result charting | Chart query results (bar, line, pie) directly from grid |
| DG03003 | Copy as SQL INSERT/UPDATE/DELETE | Generate SQL statements from selected rows |
| DG03004 | Data diff | Compare two result sets side by side |
| DG03005 | Null value handling | Display NULL clearly (empty or "NULL" label) |
| DG03006 | JSON/Array column support | Render JSON and array columns formatted/collapsible |
| DG03007 | Result set column resize to fit | Auto-fit column width to content |
| DG03008 | Result set column freeze | Freeze columns (like Excel) |
| DG03009 | Result set row number column | Show row numbers |
| DG03010 | Result set column type view | Show column data type on hover |
| DG03011 | Result set export to clipboard | Copy grid data to clipboard with configurable format |
| DG03012 | Generate CRUD SQL from grid | Auto-generate SELECT/INSERT/UPDATE/DELETE SQL for current table |

---

### 2.13 User / Role Management

| # | Feature | Mô tả |
|---|---|---|
| UM01001 | User list | Liệt kê users với role, superuser status |
| UM01002 | Create user | Tạo user với password, role, permissions |
| UM01003 | Edit user | Đổi password, role, permissions |
| UM01004 | Delete user | Xóa user |
| UM01005 | Role list | Liệt kê roles với privileges |
| UM01006 | Grant/Revoke | Cấp/revoke privileges on tables, sequences, functions |

---

### 2.14 Backup / Restore / Migration

| # | Feature | Mô tả |
|---|---|---|
| BK01001 | Database backup | Backup database/schema to file (pg_dump integration) |
| BK01002 | Database restore | Restore from backup file |
| BK01003 | Schema export | Export schema DDL to file |
| BK01004 | Data export to another DB | Transfer data between databases (database-to-database migration) |
| BK01005 | Scheduled tasks | Schedule backup/export tasks for later execution |

---

### 2.15 Packaging

| # | Feature | Mô tả |
|---|---|---|
| PK01001 | `.deb` package | Build `.deb` cho Ubuntu 22.04+ |
| PK01002 | `.AppImage` package | Build AppImage cho Ubuntu |
| PK01003 | Flatpak package | Build Flatpak (future) |