# Refinement Proposal: DBeaver-Aligned Navigation & Feature Architecture

## Problem Statement
How might we streamline DB Pro's crowded 14-button vertical activity rail into a clean, hierarchical, and discoverable workstation navigation model mapped directly to DBeaver's proven functional architecture?

---

## 1. Executive Summary & Recommended Direction

Currently, DB Pro displays **14 independent icons** on a 48px left rail (`Explorer`, `Queries`, `Files`, `Data`, `ER diagram`, `Schema workbench`, `Schema compare`, `History`, `Problems`, `Transfers`, `Monitor`, `Security`, `Saved tasks`, `Agent`, `Settings`). This causes cognitive overload, vertical clipping on standard laptop displays (800–900px height), and scatters tightly related workflows across disconnected rail buttons.

### The New Architecture (4 Primary Groups + Contextual Workstations)

Instead of 14 flat rail buttons, the navigation is restructured into **4 primary functional hubs**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Left Rail (48px)   │ Nav Drawer (240px)          │ Center Workspace         │
├────────────────────┼─────────────────────────────┼──────────────────────────┤
│ [1] Database       │ ── DATABASE NAVIGATOR ────  │ ── MAIN WORKSPACE ─────  │
│     (FolderTree)   │ ▾ Production PostgreSQL     │ Active Query Editor /    │
│                    │   ▾ public                  │ Table Workstation Tabs   │
│                    │     ▸ tables (42)           │ (Data | Columns | DDL    │
│                    │     ▸ views (8)             │  Indexes | ER Diagram    │
│                    │     ▸ routines (14)         │  Constraints | Relations)│
│                    │                             │                          │
│ [2] SQL & Projects │ ── SCRIPTS & PROJECTS ────  │ ──────────────────────── │
│     (FileCode2)    │ • Open Scratchpad           │ Bottom Dock (Expandable) │
│                    │ ▾ Project SQL Files         │ ──────────────────────── │
│                    │ ▾ Query Execution History   │ Results | Messages       │
│                    │                             │ Explain | Output Logs    │
│ [3] Tools Hub      │ ── TOOLS & MANAGEMENT ────  │                          │
│     (Boxes/Wrench) │ • Schema Compare & Sync     │                          │
│                    │ • Data Transfer / Migration │                          │
│                    │ • Server Monitor & Locks    │                          │
│                    │ • Security & Users / RLS    │                          │
│                    │ • Scheduled Tasks & Backup  │                          │
│                    │                             │                          │
│ [4] AI Copilot     │ ── AI AGENT COPILOT ──────  │                          │
│     (Sparkles)     │ Chat | Schema QA | Patches  │                          │
├────────────────────┼─────────────────────────────┴──────────────────────────┤
│ [⚙] Settings       │ Preferences | Keybindings | Appearance | Vault           │
└────────────────────┴─────────────────────────────────────────────────────────┘
```

---

## 2. DBeaver Feature-to-Code Mapping

| DBeaver Feature Category | DBeaver Native Location | DB Pro Mapping & New Location |
|---|---|---|
| **Database Navigator** | Left Primary View | **Activity 1: Database** (`Activity::Explorer`) — Connections, Schemas, Objects tree. |
| **SQL Editor & Scripts** | Projects / Scripts View | **Activity 2: SQL & Projects** (`Activity::Queries`) — Tab strip with *Scratchpads*, *Project Files*, *Execution History*. |
| **Table Data & Structure** | Table Editor Nested Tabs | **Contextual Table Tabs** — When double-clicking a table: opens single workspace tab containing nested tabs (`Data Grid`, `Columns`, `Indexes`, `Foreign Keys`, `DDL`, `Profile`). |
| **ER Diagram** | Schema/Table View Tab | **Contextual Schema Tab** + Top Toolbar Action `[Open ER Diagram]`. |
| **Schema & Data Compare** | Tools Menu / Right Click | **Activity 3 (Tools Hub)** + Context Menu (`Right click -> Compare`) + Command Palette (⌘⇧P). |
| **Data Transfer / Dump** | Tools Menu / Right Click | **Activity 3 (Tools Hub)** + Context Menu (`Right click -> Export/Import`). |
| **Server Administration** | Context Menu -> Admin | **Activity 3 (Tools Hub)** (`Monitor`, `Security/Roles`, `FDW`, `Event Triggers`). |
| **Execution Log & Diagnostics** | Bottom Output Dock | **Bottom Output Panel** (`Results`, `Messages`, `Explain Plan`, `Problems/Diagnostics`). |
| **AI Smart Assistant** | DBeaver AI / Copilot Panel | **Activity 4: AI Copilot** (Right or Left Agent Dock). |

---

## 3. Dual-Access Discovery: Context Menu + Quick Actions

To satisfy both power users and quick discoverability (selected as *"Cả 2"*):

1. **Context Menu & Command Palette (⌘⇧P)**:
   - Right-clicking any Connection, Schema, or Table exposes immediate tools:
     - `Compare Schema…` / `Compare Data…`
     - `Export Data…` / `Import Data…`
     - `View ER Diagram`
     - `Server Administration` (`Sessions`, `Locks`, `Users & Privileges`)
   - All tools registered in the global Command Palette (`⌘⇧P`).

2. **Quick Action Bar / Tool Header**:
   - Each workspace surface displays a unified top action header:
     - Table view: `[View ER]` `[Export CSV]` `[Generate DDL]`
     - Query view: `[Run ⌘↵]` `[Explain]` `[Format]` `[Ask AI]` `[Transaction Toggle]`

---

## 4. Key Assumptions to Validate

1. **Assumption 1**: Developers prefer a 4-icon rail with categorized drawers over scrolling through 14 vertical icons.
   - *Validation*: Verify on 1280×800 and 1440×900 display viewports; ensure zero vertical icon overflow.
2. **Assumption 2**: Consolidating table-specific surfaces (`Data`, `Structure`, `DDL`, `Indexes`, `Profile`) into sub-tabs of the Table Workstation matches user mental models from DBeaver/DataGrip.
   - *Validation*: Double-clicking a table in Explorer opens the Table tab with immediate access to Data and Structure without leaving the tab.
3. **Assumption 3**: Moving administrative tools (`Monitor`, `Security`, `Transfers`, `Compare`) into a unified "Tools Hub" drawer reduces visual clutter while keeping full capability accessible.

---

## 5. MVP Implementation Scope

### What's In Scope:
- **Refactor ActivityBar**: Reduce rail buttons from 14 to **4 main activities + Settings**:
  1. `Database` (Explorer)
  2. `SQL & Scripts` (Queries + Files + History)
  3. `Tools & Management` (Compare + Transfers + Monitor + Security + Tasks + FDW)
  4. `AI Copilot` (Agent)
  5. `Settings` (Preferences)
- **Nested Drawers**:
  - `SQL & Scripts` sidebar drawer offers segmented tabs: *Open Queries*, *Recent Files*, *Snippets*, *History*.
  - `Tools` sidebar drawer offers categorized tool cards: *Schema & Migration*, *Data Import/Export*, *Administration & Security*.
- **Right-Click Context Menus**: Wire Explorer tree nodes to launch tools directly.
- **Top Bar Quick Action Presets**: Maintain immediate 1-click access to Run, Explain, ER, Export.

### What's Out of Scope (Not Doing):
- **Not rewriting the backend services**: `MonitoringService`, `ObjectMutationService`, `DataDiffService`, and `QueryService` remain 100% untouched.
- **Not removing any capabilities**: All 14 feature sets remain fully supported, simply re-homed into a clean hierarchy.
- **Not forcing single-panel layout**: Keep sidebars resizable and collapsible.

---

## 6. Verification & Quality Gates

- [ ] Zero vertical scrolling required on the 48px left rail at 1280×800.
- [ ] Switching between Database, Scripts, Tools, and Agent works seamlessly.
- [ ] All 1,787 existing workspace automated tests pass cleanly.
- [ ] Clean code score remains ≥94/100 (Grade A) with 0 errors.
