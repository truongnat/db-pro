# Agent evidence — Query Editor large-schema audit

## 1. Claim

Agent Codex root; research/performance audit lane, task Done. Feature state PLANNING for proposed remediation. Baseline exact SHA 3dd988e2ff06ac71942c7ec331acf66389d3c5cb. Main; PR n/a. Scope: owner's PostgreSQL >500-table crash/Query lag audit. Production source changes: none. Existing dirty checkout preserved.

## 2. Progress checkpoint

Completed source trace, public-path synthetic measurements, twice reproduced Unicode panic, evidence fingerprints and recommendations. Actual company startup crash, native popup frame-time, provider latency and OOM evidence pending. Task Done means audit report delivered, not bug fixed.

## 3. Implementation handoff / review request

No implementation. Five findings: 2 P1 (UTF-8 panic and unbounded completion pipeline), 3 P2 (result-frame clone, UI index build, repeated metadata scans). Proposed minimal fixes and acceptance boundaries in FINDINGS.md. Keep provider-qualified identities, request generations and editor semantics. No credentials, schema dump or company access collected.

## 4. Review outcome

Self-review verdict BLOCK for claiming large-schema Query readiness or crash resolution until P1 paths are fixed. Introduced findings by this audit: P0=0 / P1=0 / P2=0 (no product patch). Audited baseline defects/risks: P0=0 / P1=2 / P2=3. Exact committed behavior is asserted only on audited unchanged paths at SHA above; current cached library/source fingerprints in manifest. Skipped gates are listed in VERIFICATION.md, not marked passed. Independent review/CI n/a.

## 5. Research / audit handoff

Primary evidence: current local code, optimized public-path probe, source equality to exact SHA, measurements.log and manifest.json. Source date 2026-10-06. PostgreSQL user-reported affected; shared frontend findings also apply to SQLite paths, live tests pending independently. No remote PR coverage check or external web research needed for this local audit. Lessons recorded locally; global memory unchanged.

## 6. Tổng kết (Vietnamese summary)

Đã tái hiện panic UTF-8 và đo autocomplete không scope vượt frame budget ở 500–1000 bảng. Popup chưa virtualize, Query clone result mỗi frame, schema index xây trên UI thread và mapper quét metadata nhiều lần. Completion có alias trên SQL ngắn vẫn nhanh; chưa có bằng chứng chốt nguyên nhân sập lúc load DB công ty. Audit chưa sửa product code; ưu tiên sửa panic và pipeline completion trước, rồi ownership/index/metadata và runtime trace thực tế.
