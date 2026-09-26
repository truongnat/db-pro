# DB Pro Native — Design System (from Stitch)

Source: Stitch Project `projects/1651725598433404641` (DB Pro — Native Database IDE)

## Product
DB Pro is a native desktop Database IDE for engineers and database operators. Its primary work is connecting to PostgreSQL and SQLite, inspecting schemas, writing SQL, running queries, and reviewing results. Prioritize information density, precision, legibility, and predictable controls over marketing-style visuals. This Stitch project is a visual design reference; the shipped UI is native Rust/egui.

## Visual direction
Dark-first, restrained workstation interface. Familiar developer-tool density, but with crisp hierarchy and deliberate spacing. Keep the canvas quiet so code, query status, and tabular results carry attention. Build surfaces from planes and separators, not floating SaaS cards. Support a corresponding light theme later, but all initial components use dark mode.

## Color tokens
- App background: `#212121`
- Panel surface: `#2A2A2A`
- Editor surface: `#181818`
- Elevated surface: `#303030`
- Hover surface: `#363636`
- Active surface: `#3D3D3D`
- Subtle border: `#323232`
- Default border: `#414141`
- Strong border: `#505050`
- Primary text: `#ECECEC`
- Secondary text: `#B9B9B9`
- Tertiary text: `#8D8D8D`
- Disabled text: `#666666`
- Accent blue: `#339CFF`
- Accent hover: `#66B5FF`
- Accent soft: `#00284D`
- Success: `#22C55E`
- Warning: `#F59E0B`
- Danger: `#EF4444`
- Info: `#3B82F6`

### Code syntax colors:
- keyword: `#82AAFF`
- string: `#98C379`
- number: `#E5C07B`
- comment: `#777777`
- type: `#C792EA`
- function: `#61D6D6`

## Typography
Use Inter for navigation, labels, controls, and UI text. Use JetBrains Mono for SQL, identifiers, numeric result values, and keybindings.
Compact hierarchy:
- screen title: 18/24 semibold
- section heading: 13/20 semibold
- body: 13/20 regular
- labels: 12/16 medium
- supporting text: 11/16
- code: 12/20

## Shape and spacing
- Base spacing unit: 4px
- Controls height: compact (28px)
- Toolbar height: ~36px
- Radius: 4px for dense controls, 6px only for dialogs and menus
- Pane separators: 1px
- Panel padding: 12px
- Table row height: 28px
- Editor line height: 20px

## Screens in Stitch (19 screens)
1. DB Pro Component Library — SQL Editor & Results Grid
2. DB Pro — Execution History (F1 Unified Execution History)
3. DB Pro Component Library (Main)
4. DB Pro — Queries Activity (F1 Safe Open & Connection Context)
5. DB Pro — SQL Editor & Results Grid (Native Theme Fixed)
6. DB Pro — Data Activity (F1 Stable Table Reference Identity)
7. DB Pro — ER Diagram (audit schema)
8. DB Pro — SQL Editor & Results Grid (Native Theme)
9. DB Pro — Schema Compare (F1 Block Incomplete Migration DDL)
10. DB Pro — SQL Editor & Results Grid (Native Theme)
11. DB Pro Component Library — Inputs, Selection & Navigation
12. DB Pro — Execution History (Proposed F1)
13. DB Pro — ER Diagram (F1 Schema-Scoped Graph Refresh)
14. DB Pro Component Library — Feedback, Dialogs & Commands
15. DB Pro — Rebind a Missing Table Reference
16. DB Pro — Files Workspace (F1 Trust & File Mutation Review)
17. DB Pro — Queries Activity (F1 Source Context Unavailable)
18. DB Pro — Explorer (F1 Object Filter Workbench)
19. DESIGN.md
