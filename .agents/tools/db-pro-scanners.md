# Scanner nội bộ db-pro

Công cụ có script, không phải MCP. Chạy từ root repo.

| Công cụ | Lệnh | Mục đích |
|---|---|---|
| Perf audit | `bash .skills/perf-audit/scripts/perf-scan.sh` | Đo/scan perf native UI |
| Clean code | `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | Smell scan trước PR |
| UI consistency | `python3 .skills/ui-consistency-audit/scripts/ui_consistency_scan.py` | Inventory Select / contract drift |

Cấu hình clean-code: `clean-code.config.json`. Chi tiết skill: `.skills/*/SKILL.md`.
