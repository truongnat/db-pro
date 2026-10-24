# `.agents/` — nguồn chuẩn

Thư mục này chứa workflow, quy tắc, điểm mở rộng, script, template và memory lâu dài của repository.

- `AGENTS.md`: hợp đồng làm việc cơ bản cho agent.
- `rules/`: quy chuẩn và quy trình phát triển phần mềm; cần tùy chỉnh `project.md` cho từng repository.
- `skills/`: năng lực theo từng loại tác vụ, được đăng ký với agent đang sử dụng.
- `mcp/`: hướng dẫn cấu hình MCP và quyền truy cập.
- `tools/`: công cụ hỗ trợ có hành vi xác định.
- `scripts/`: script có thể thực thi.
- `template/`: template Markdown có thể tái sử dụng.
- `memory/`: knowledge base có thể tìm kiếm. **Mirror bytes và JSON pipeline không commit** (xem `memory/.gitignore`); sau clone: `scripts/bootstrap-agents-memory.sh` (hoặc VERIFICATION §9). Policy: `docs/ARTIFACT_POLICY.md`.

Không mặc định rằng công cụ sẽ tự phát hiện mọi thư mục. Hãy đăng ký skill/MCP hoặc cấu hình adapter cho file chỉ dẫn theo cơ chế riêng của agent đó.

**db-pro:** `skills/` — **13 skill gốc vòng đời** (blueprint 3: `software-lifecycle`, `memory-curation`, …) + ~56 vendored (`skills-lock.json`). Validate: `.agents/scripts/validate-skills.sh`. Scanner `.skills/`: `tools/db-pro-scanners.md`. Workflow memory: `AGENTS.md` (thư mục này); sản phẩm: `AGENTS.md` (root).
