REDESIGN CORRECTION — CURRENT UI IS REJECTED

The previous redesign attempt FAILED visually.

Do not treat the current UI as an acceptable foundation that only needs polishing.

The screenshot/current implementation still looks almost identical to the old DB PRO:
- same overall layout proportions
- same explorer presentation
- same button-heavy egui appearance
- same outlined controls
- same large empty surfaces
- same weak visual hierarchy
- same prototype/tooling feeling

This task is NOT another polish pass.

I want a STRUCTURAL VISUAL REDESIGN.

The existing database/domain functionality should be preserved, but the PRESENTATION LAYER and WORKSPACE COMPOSITION may be substantially rewritten.

==================================================
PRIMARY REQUIREMENT
==================================================

When I launch the application after this task, the visual difference must be IMMEDIATELY OBVIOUS even before interacting with anything.

If a before/after screenshot looks approximately the same at first glance, this task has FAILED.

Do NOT optimize for minimal diff.

Do NOT preserve UI structures simply because they already exist.

Do NOT respond with a plan only.

Audit briefly, then IMPLEMENT the redesign.

==================================================
TARGET EXPERIENCE
==================================================

Build a premium native developer tool inspired by the quality and restraint of modern Codex-style desktop applications.

This is NOT:
"DBeaver with rounded buttons."

This is:

A modern database workbench with:
- professional desktop density
- excellent hierarchy
- minimal chrome
- contextual actions
- large productive workspace
- elegant navigation
- strong typography
- subtle surfaces
- excellent dark mode
- first-class SQL/data workflows

DBeaver should influence CAPABILITIES only.

Do NOT imitate DBeaver's old desktop visual design.

==================================================
1. DARK MODE FIRST
==================================================

The current bright white interface is rejected as the primary visual target.

Implement a polished DARK theme as the primary/default design.

Use approximately this hierarchy conceptually:

app background       near-black
sidebar              slightly elevated
workspace            elevated surface
secondary surface    another subtle elevation
border               extremely subtle
primary text         high contrast
secondary text       medium contrast
muted metadata       low contrast
accent               restrained violet/indigo

Do not make everything pure black.

Create depth using subtle surface differences.

Avoid visible borders around every component.

The app should look premium even with NO DATABASE CONNECTED.

==================================================
2. REBUILD THE TOP CHROME
==================================================

Current top area is too similar to a web application.

Replace it with a compact native workspace header.

LEFT:

DB PRO
workspace / connection context

CENTER:
workspace tabs should visually integrate into the application rather than looking like browser buttons.

RIGHT:

Quick Open
Agent
small utility icons

Remove unnecessary rectangles.

Buttons without strong importance should often become icon actions with hover states.

The header should feel approximately 40-44px high and dense.

==================================================
3. REDESIGN LEFT NAVIGATION COMPLETELY
==================================================

Current Explorer is rejected.

Do NOT render connection like:

[ Production ] PostgreSQL [ Edit ] [ X ]

Do NOT render a large purple "+ New connection" button permanently.

Instead create a two-level navigation system.

LEVEL 1: ACTIVITY RAIL

Very narrow vertical rail containing icons:

Database
Queries
History
Transfers
Monitor
Agent

Settings anchored near bottom.

Selected activity should use a subtle accent surface, NOT a giant colored button.

LEVEL 2: EXPLORER

Header:

DATABASE
                       +   ...

Then:

● Production
  PostgreSQL

  ▾ postgres
    ▾ public
      ▸ Tables              24
      ▸ Views                3
      ▸ Functions           12
      ▸ Sequences            4

Connections must look like tree/workspace entities, NOT form controls.

Connection actions belong in:
- hover actions
- context menu
- overflow menu

Add Connection should be a compact icon/action near Explorer header or an elegant empty-state action.

Explorer must feel similar to a professional code editor/file navigator.

==================================================
4. WORKSPACE TABS MUST FEEL NATIVE
==================================================

Current tabs still resemble independent rounded buttons.

Redesign tabs into a real IDE tab strip.

Example:

Welcome | Query 1 ● | ER Diagram | customers | +

Active tab:
- slightly different surface
- stronger text
- subtle bottom/top indicator if needed

Inactive:
- quiet
- borderless
- compact

Close icon should appear naturally, preferably on hover where appropriate.

Unsaved state uses a small dot.

No large purple tab backgrounds.

==================================================
5. CREATE A STRONG EMPTY WORKSPACE
==================================================

Current ER Diagram screen has a giant meaningless empty rectangle.

This is rejected.

An empty workspace should still look intentional.

For ER Diagram, create a proper infinite-canvas style surface:
- subtle grid/dot background
- canvas fills available workspace
- floating compact zoom controls
- compact canvas toolbar
- tasteful centered empty state

Example:

            ◇
       No tables yet

Drag tables from Explorer
or select tables to build a relationship diagram.

      [ Select tables ]

Do NOT put this inside a giant bordered white card.

The CANVAS itself is the workspace.

==================================================
6. REMOVE "EGUI DEFAULT WIDGET" FEEL
==================================================

Audit every visible control.

The final UI must not look like default egui widgets arranged with horizontal/vertical layouts.

Create reusable styled components for:

IconButton
ToolbarButton
Tab
TreeRow
SearchInput
Badge
PanelHeader
ContextMenu
Popover
StatusItem
EmptyState
SplitHandle

Control heights, padding, radius and states must be centralized.

Use custom visuals/frame/layout where required.

If default egui rendering prevents the target quality, override it.

==================================================
7. TYPOGRAPHY AND SPACING
==================================================

Current hierarchy is weak.

Create clear typography levels:

Application identity
Workspace title
Panel heading
Primary UI text
Secondary UI text
Metadata
Code

Desktop density matters.

Prefer:

4
6
8
12
16
24

spacing increments.

Do NOT waste large vertical space on toolbars.

Do NOT center everything inside large containers.

Metadata should be small and quiet.

==================================================
8. SQL QUERY WORKSPACE
==================================================

Redesign Query into this conceptual composition:

┌───────────────────────────────────────────────┐
│ Query 1                 postgres / public  ▾  │
│                              ▶ Run    ···     │
├───────────────────────────────────────────────┤
│ 1  SELECT *                                  │
│ 2  FROM customers                            │
│ 3  LIMIT 100;                                │
│                                              │
│                                              │
├───────────────────────────────────────────────┤
│ Results   Messages   Explain   History        │
├───────────────────────────────────────────────┤
│ professional result grid                     │
└───────────────────────────────────────────────┘

The SQL editor gets maximum useful space.

Do NOT restore the previous huge toolbar containing:

Search
A-
A+
Completion
Snippets
font size
folder
new folder
save
etc.

These belong in:
command palette
overflow menus
keyboard shortcuts
editor settings
context menus

Primary visible action is RUN.

==================================================
9. RESULT GRID
==================================================

The result area must look like a serious database tool.

Header should be compact.

Columns should have:
- clear separators
- type metadata where useful
- sorting state
- resizing
- selection

Rows should have subtle hover/selection.

Dirty edits should be visually obvious but restrained.

When changes exist:

2 pending changes          Discard     Apply

Do NOT expose manual row-mutation forms.

==================================================
10. STATUS BAR
==================================================

Create a compact native IDE status bar.

Example:

● Production    PostgreSQL 16    postgres    public      23ms    UTF-8

Connection state must be immediately understandable.

Do not waste space.

==================================================
11. MOTION / INTERACTION
==================================================

Add subtle transitions where egui reasonably supports them:

panel reveal
hover state
selection
Agent panel
command palette
loading
connection state

No flashy web animations.

Interaction should feel responsive and precise.

==================================================
12. IMPLEMENT A DESIGN SYSTEM
==================================================

Before individually styling screens, establish shared tokens/components.

Suggested structure conceptually:

ui/
  theme/
    colors
    typography
    spacing
    metrics
  components/
    icon_button
    tabs
    tree
    search
    toolbar
    badge
    status_bar
    empty_state
    data_grid
  layout/
    app_shell
    activity_rail
    explorer
    workspace
    bottom_panel

Adapt names to the existing repository conventions.

Do NOT create architecture duplication if equivalent structures already exist.

==================================================
13. REMOVE VISUAL DEBT
==================================================

While implementing, actively remove old presentation patterns that conflict with the redesign.

Specifically search for and eliminate:

large purple CTA buttons used for routine IDE actions
outlined rectangle around every control
connection Edit/X buttons permanently visible
browser-like rounded tabs
large bordered empty panels
unnecessary toolbar controls
duplicate navigation/actions
hardcoded spacing/colors
screen-specific button styling

Do not leave old and new design systems mixed together.

==================================================
VISUAL ACCEPTANCE TEST
==================================================

Before declaring completion:

Run the application and inspect the actual rendered UI.

Capture/inspect at minimum:

1. No connection / empty state
2. Connected database explorer
3. SQL Query workspace
4. Query results
5. ER Diagram
6. Agent panel if available

Compare them mentally against the PREVIOUS UI.

Ask:

"Would someone immediately believe this is a different generation of the application?"

If NO, continue redesigning.

The task is NOT complete because code compiles.

The task is NOT complete because tokens were introduced.

The task is NOT complete because spacing changed.

The task is complete only when the RUNNING APPLICATION has materially changed.

==================================================
FINAL PRODUCT CHARACTER
==================================================

The UI should communicate:

PREMIUM
NATIVE
TECHNICAL
DENSE
CALM
FAST
POWERFUL

Not:

WEB ADMIN
FORM BUILDER
DEFAULT EGUI
BOOTSTRAP
OLD DATABASE CLIENT
PROTOTYPE

Keep the existing product functionality.

Replace its visual language.

The desired reaction when opening DB PRO should be:

"This looks like a modern native developer tool."

not:

"This looks like the same DB tool with slightly different spacing."
