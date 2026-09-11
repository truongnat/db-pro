GOAL: REDESIGN DB PRO INTO A PREMIUM NATIVE DATABASE IDE

You are working on DB PRO, a native desktop database management application built entirely with Rust + egui.

Your goal is to redesign and evolve the current application into a premium, professional database IDE.

The product should combine:

- DBeaver-level database capabilities and information density.
- Codex-class modern desktop interaction quality and visual polish.
- Native IDE/workbench productivity similar to high-quality developer tools.
- A powerful contextual AI/Agent experience designed specifically for database work.

IMPORTANT:
Do NOT simply reskin the existing UI.
Do NOT copy DBeaver's visual design.
Do NOT turn the application into a web dashboard.

DBeaver is primarily a CAPABILITY reference.
Modern applications such as Codex are an INTERACTION, UX, and visual-quality reference.

The final product should feel like a modern native database IDE designed from first principles.

==================================================
1. PRODUCT DESIGN PRINCIPLE
==================================================

Use this product formula:

DBeaver feature depth
+ modern Codex-quality interaction
+ native desktop IDE density
+ contextual database AI
= DB PRO

The application must remain highly productive for professional developers, DBAs, and power users.

Prioritize:
- information density
- keyboard-first workflows
- fast navigation
- contextual actions
- minimal visual noise
- strong hierarchy
- consistent components
- large working areas for SQL and data
- discoverability without toolbar clutter

Avoid:
- excessive cards
- excessive rounded rectangles
- giant buttons
- web-dashboard layouts
- unnecessary borders
- duplicated actions
- permanently visible secondary controls
- excessive whitespace that reduces IDE productivity

==================================================
2. APPLICATION SHELL
==================================================

Redesign the application around a stable desktop workbench.

Primary layout:

Top Application Bar
--------------------------------
Side Activity Rail
Database/Object Explorer
Main Workspace
Contextual Right Panel (optional)
Bottom Panel
Status Bar

The main workspace must support multiple tabs.

Examples:

Query.sql
customers
orders
ER Diagram
Query Plan

The application shell must support resizable split panes.

Panel state and dimensions should be persistent.

The workspace must remain visually stable when switching between database objects.

==================================================
3. ACTIVITY RAIL + EXPLORER
==================================================

Create a compact vertical activity rail.

Potential sections:

Database
Queries
History
Transfers
Monitoring
Agent
Extensions
Settings

The adjacent Explorer changes according to selected activity.

Database Explorer should support hierarchical navigation:

Connection
  Database
    Schema
      Tables
      Views
      Materialized Views
      Functions
      Procedures
      Sequences
      Types
      Triggers

Objects should be expandable without unnecessary cards.

Connection information should be represented compactly.

Use:
icons
typography
indentation
hover states
selection states
small metadata badges

instead of large bordered containers.

Context menus must expose object-specific actions.

==================================================
4. SQL EDITOR AS THE PRIMARY WORKSPACE
==================================================

The SQL editor is one of the most important surfaces in the product.

It must visually dominate the workspace instead of appearing like a large textarea.

Support architecture for:

- syntax highlighting
- line numbers
- current-line indication
- selection execution
- SQL formatting
- schema-aware autocomplete
- table/column suggestions
- diagnostics
- multiple query tabs
- query execution state
- execution timing
- keyboard shortcuts
- query history
- connection/schema context
- multiple result sets

Keep the editor toolbar extremely compact.

Primary action:

Run

Secondary actions should use icons, menus, keyboard shortcuts, command palette, or contextual actions.

Do NOT permanently expose controls such as:

font size
completion toggle
snippet manager
folder fields
format settings

as large toolbar buttons.

==================================================
5. RESULT DATA GRID
==================================================

Replace prototype-style row mutation controls with a professional editable DataGrid.

The result grid must become a first-class subsystem.

Support:

- large datasets
- column resizing
- sorting
- filtering
- column pinning where appropriate
- cell selection
- copy/paste
- null representation
- editable cells
- dirty-row state
- staged mutations
- apply/discard changes
- pagination or virtualized loading
- row count
- execution duration
- multiple result tabs
- export

Bottom workspace tabs may include:

Results
Messages
Explain
Query Plan
History

Database editing should happen naturally inside the grid.

Never require users to manually enter:
table
primary-key column
primary-key value
column
new value

for normal row editing.

==================================================
6. DATABASE OBJECT WORKSPACE
==================================================

Opening a table should create a workspace tab.

Example:

customers

Data | Structure | Indexes | Foreign Keys | Constraints | DDL | Dependencies

Structure view should clearly represent:

primary keys
foreign keys
column type
nullable
default
unique
indexes
references

Allow contextual inspection without navigating through multiple modal dialogs.

Use optional contextual inspector panels for detailed metadata.

==================================================
7. COMMAND PALETTE
==================================================

Implement a first-class command palette.

Shortcut concept:

Cmd/Ctrl + K

Commands may include:

New SQL Query
Open Table
Switch Connection
Run Query
Format SQL
Explain Query
Export Results
Import Data
Search Database Object
Show Query History
Toggle Panel
Ask Agent

As DB PRO grows toward DBeaver-level capability, the command palette must prevent the UI from becoming toolbar-heavy.

==================================================
8. AI / AGENT EXPERIENCE
==================================================

Agent must NOT simply be a generic chatbot attached to the application.

It should understand database context.

Context may include:

active connection
database engine
current schema
selected table
selected columns
current SQL
query results
database metadata
EXPLAIN output
errors

Possible contextual actions:

Explain query
Fix SQL
Optimize query
Generate SQL
Explain database object
Find related tables
Investigate error
Analyze query plan
Suggest index
Generate migration
Explore schema

Agent suggestions should appear near the relevant context when appropriate.

Example:

User selects SQL -> contextual "Ask / Explain / Optimize".

Agent may propose SQL changes as a diff.

Potential actions:

Show Diff
Apply
Run
Run EXPLAIN

Never perform destructive database operations silently.

==================================================
9. VISUAL SYSTEM
==================================================

Create a centralized egui design system before continuing screen-by-screen styling.

Define reusable tokens for:

colors
surface levels
text hierarchy
spacing
radius
border
control heights
typography
icon sizes
hover
active
selected
focus
disabled
warning
error
success

Prefer subtle surface layering over heavy borders.

Use compact desktop spacing.

Recommended spacing scale:

4 / 6 / 8 / 12 / 16 / 24

Typography should clearly distinguish:

primary
secondary
muted
metadata
code

The visual result should feel calm, precise, technical, premium, and native.

Both dark and light themes should be structurally supported, even if one theme is implemented first.

==================================================
10. COMPONENT ARCHITECTURE
==================================================

Do not scatter visual constants and custom drawing logic throughout screens.

Build reusable UI primitives/components.

Examples:

AppShell
ActivityRail
Explorer
SplitView
Tabs
Toolbar
IconButton
TreeView
DataGrid
StatusBar
CommandPalette
ContextMenu
Popover
Inspector
EmptyState
LoadingState
ErrorState

Screens should compose these primitives rather than reimplement their own visual language.

Respect existing Rust architecture where reasonable.

Audit the current codebase before restructuring anything.

Do not rewrite working database/domain logic merely to redesign UI.

==================================================
11. DBeaver-CLASS CAPABILITY DIRECTION
==================================================

The architecture should allow DB PRO to progressively support:

connection management
schema exploration
SQL editing
data browsing/editing
table structure
indexes
foreign keys
constraints
query plans
transactions
import/export
query history
sessions
locks
database monitoring
schema comparison
data comparison
ER diagrams
SSH/SSL
multiple database engines
extensions/providers

Do NOT attempt to implement every feature immediately.

The redesign must create a scalable foundation where these capabilities can be added without degrading UX.

==================================================
12. EXECUTION STRATEGY
==================================================

Before modifying code:

1. Audit the current UI architecture.
2. Identify reusable components.
3. Identify prototype UI that should be removed.
4. Identify database/domain logic that must remain untouched.
5. Produce a concise implementation plan.

Then execute incrementally:

P0 - Design tokens + application shell
P1 - Activity rail + database explorer
P2 - Workspace tabs + SQL editor shell
P3 - Professional result DataGrid
P4 - Database object workspace
P5 - Command palette
P6 - Contextual Agent UX
P7 - Advanced database tooling

After each phase:
- compile
- run tests
- verify existing functionality
- visually inspect the result
- fix regressions before proceeding

==================================================
ACCEPTANCE CRITERIA
==================================================

The redesign succeeds when:

1. The application no longer looks like a web admin or egui prototype.
2. SQL editor and result grid dominate the working experience.
3. Common workflows require fewer visible controls.
4. Database navigation is compact and scalable.
5. Panels/tabs behave like a professional desktop IDE.
6. Visual styling is consistent through reusable tokens/components.
7. Adding new database capabilities does not require adding toolbar clutter.
8. Existing working database functionality remains intact.
9. AI feels integrated with database workflows rather than bolted on.
10. The overall experience can reasonably be described as:

"A premium native database IDE with DBeaver-class capability depth and modern Codex-class interaction quality."

Do not optimize only for screenshots.

Optimize for the application users will operate for hours every day.
