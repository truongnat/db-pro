# Dead Code Analysis Report

**Date**: 2026-08-13  
**Tool**: knip@6.27.0

## Summary

- **45 unused files** (~15-20KB estimated)
- **33 unused dependencies** (significant bundle impact)
- **148 unused exports** (API surface, keep for now)
- **70 unused types** (keep for type definitions)

## Unused Files (Safe to Delete)

### UI Components (unused shadcn components)
- `src/components/ui/avatar.tsx`
- `src/components/ui/card.tsx`
- `src/components/ui/progress.tsx`
- `src/components/ui/separator.tsx`
- `src/components/ui/skeleton.tsx`
- `src/components/ui/tabs.tsx`

### IDE Components (legacy/unused)
- `src/commons/components/ide/ide-empty-state.tsx`
- `src/commons/components/ide/ide-error-state.tsx`
- `src/commons/components/ide/ide-icon-button.tsx`
- `src/commons/components/ide/ide-panel-header.tsx`
- `src/commons/components/ide/ide-resize-handle.tsx`
- `src/commons/components/ide/ide-section-tabs.tsx`
- `src/commons/components/ide/ide-surface.tsx`
- `src/commons/components/ErrorBoundary.tsx`

### Feature Modules (unused/planned)
- `src/modules/backup/components/backup-dialog.tsx`
- `src/modules/backup/components/restore-dialog.tsx`
- `src/modules/connection/pages/connections-page.tsx`
- `src/modules/data-grid/components/chart-config-dialog.tsx`
- `src/modules/data-grid/components/chart-view.tsx`
- `src/modules/export/components/import-dialog.tsx`
- `src/modules/query/components/query-context-strip.tsx`
- `src/modules/query/components/transaction-bar.tsx`
- `src/modules/schema/components/cross-connection/*` (5 files)
- `src/modules/schema/components/index-list.tsx`
- `src/modules/user-management/components/*` (4 files)
- `src/modules/user-management/pages/user-management-page.tsx`

### Services & Utils
- `src/commons/di/index.ts`
- `src/commons/utils/clipboard.ts`
- `src/commons/utils/index.ts`
- `src/commons/utils/validation.ts`
- `src/modules/connection/services/connection.agent.ts`
- `src/modules/data-grid/services/data-grid.agent.ts`
- `src/modules/export/services/export.agent.ts`
- `src/modules/schema/services/schema.agent.ts`
- `src/modules/user-management/services/user-management.agent.ts`
- `src/modules/unified-grid/index.ts`

### Scripts
- `scripts/generate-fixtures.mjs`

## Unused Dependencies (Safe to Remove)

### Radix UI (likely unused shadcn components)
- `@radix-ui/react-accordion`
- `@radix-ui/react-alert-dialog`
- `@radix-ui/react-aspect-ratio`
- `@radix-ui/react-avatar`
- `@radix-ui/react-checkbox`
- `@radix-ui/react-collapsible`
- `@radix-ui/react-context-menu`
- `@radix-ui/react-dropdown-menu`
- `@radix-ui/react-hover-card`
- `@radix-ui/react-label`
- `@radix-ui/react-menubar`
- `@radix-ui/react-navigation-menu`
- `@radix-ui/react-popover`
- `@radix-ui/react-progress`
- `@radix-ui/react-scroll-area`
- `@radix-ui/react-select`
- `@radix-ui/react-separator`
- `@radix-ui/react-slider`
- `@radix-ui/react-slot`
- `@radix-ui/react-switch`
- `@radix-ui/react-tabs`
- `@radix-ui/react-toast`
- `@radix-ui/react-toggle`
- `@radix-ui/react-toggle-group`
- `@radix-ui/react-tooltip`

### Other
- `@hookform/resolvers`
- `date-fns`
- `input-otp`
- `react-day-picker`
- `react-hook-form`
- `react-is`
- `recharts`
- `sonner`

### Dev Dependencies
- `@types/cytoscape` (cytoscape has built-in types)

## Impact Estimate

### Bundle Size Reduction
- Removing unused files: ~15-20KB
- Removing unused dependencies: **200-400KB** (estimated)
  - recharts: ~150KB
  - date-fns: ~50KB
  - Radix UI components: ~100KB total
  - Others: ~50KB

### Total Estimated Savings: **250-450KB** (10-20% reduction)

## Action Plan

1. ✅ Delete 45 unused files
2. ✅ Remove 33 unused dependencies from package.json
3. ✅ Run npm install to update lockfile
4. ✅ Rebuild and measure actual impact
5. ⏸️ Do NOT remove unused exports (may be public API)
6. ⏸️ Do NOT remove unused types (type definitions)

## Warnings

- Some Radix UI components may be used through shadcn/ui wrappers
- Test thoroughly after removal
- Some "unused" features may be planned for future use
