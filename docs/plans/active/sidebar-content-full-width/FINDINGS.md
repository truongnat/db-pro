# Findings — sidebar-content-full-width

## P0 / P1

None open for this fix scope after the header hover fix.

## Evidence (pre-fix)

1. Screenshot crop: drag line ~445px, `New query` chrome ~360px, tree/`Failed:` ink ~200px.
   Tree was not filling the sidebar content width.
2. Workspace selector painted `surface_hover` *after* the label → hover wiped the name
   to a blank wash ("trắng xóa").

## Fixes in this branch

- Content rect + drag line locked to `sidebar_width`
- Explorer ScrollArea / tree rows forced to content width
- Header selector: allocate → hover wash → text (correct paint order)
- Removed duplicate Plus on the explorer filter row (New Connection stays in header)

## Residual

Runtime screenshot evidence still pending after native rebuild.
