import type { ActionAuditEvent } from "./types";

/**
 * Lightweight audit log for action executions.
 *
 * Emits events at action start, completion, error, and cancellation.
 * Consumers (debug panel, file logger, remote telemetry) subscribe
 * via `onAuditEvent`.
 */

type AuditListener = (event: ActionAuditEvent) => void;

const listeners = new Set<AuditListener>();

/** Maximum events kept in the in-memory buffer. */
const MAX_BUFFER = 500;

/** Ring buffer of recent audit events (newest last). */
const buffer: ActionAuditEvent[] = [];

/** Subscribe to audit events. Returns an unsubscribe function. */
export function onAuditEvent(listener: AuditListener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** Emit an audit event to all listeners and store it in the buffer. */
export function emitAuditEvent(event: ActionAuditEvent): void {
  buffer.push(event);
  if (buffer.length > MAX_BUFFER) {
    buffer.splice(0, buffer.length - MAX_BUFFER);
  }
  for (const listener of listeners) {
    try {
      listener(event);
    } catch (err) {
      console.error("[ActionAudit] Listener error:", err);
    }
  }
}

/** Get a copy of the recent audit event buffer. */
export function getAuditBuffer(): readonly ActionAuditEvent[] {
  return [...buffer];
}

/** Clear the audit buffer (test / reset). */
export function clearAuditBuffer(): void {
  buffer.length = 0;
}
