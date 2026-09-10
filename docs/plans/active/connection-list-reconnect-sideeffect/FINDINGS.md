# Findings — QA-P1-09 Connection-list query reconnect side-effect

## Overview
**Severity:** P1
**Area:** Connection lifecycle / React Query
**Files:** `frontend/src/modules/connection/queries/connection.queries.ts`

## Evidence
In `connection.queries.ts`, `useConnectionList` calls `restoreSession(connections)` inside `queryFn`.
Because React Query invalidations occur whenever connection operations (create, update, connect, disconnect, delete) succeed, `useConnectionList` refetches frequently.

## Failure Scenario
1. App starts and restores active sessions.
2. User performs a connection action or React Query invalidates `["connections"]`.
3. `useConnectionList` refetches and re-runs `restoreSession`.
4. If session restoration state is re-triggered or reset, redundant reconnect requests (`service.connect()`) are sent for all previously active connection IDs, causing connection status flickering, race conditions, or unnecessary socket/network connections.

## Solution
Decouple `restoreSession` from `useConnectionList`. `useConnectionList` will only fetch saved connections and update the store, while session restoration is managed by a dedicated one-shot startup mechanism.
