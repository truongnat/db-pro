import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

import { TriggerManager } from "../components/trigger-manager";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("@/commons/stores", () => ({
  useConnectionStore: (selector: (s: { connections: { id: string; driver: string }[] }) => unknown) =>
    selector({ connections: [{ id: "conn-1", driver: "postgres" }] }),
}));

const mockTriggers = [
  {
    name: "log_update",
    tableName: "users",
    schema: "public",
    timing: "AFTER",
    event: "UPDATE",
    definition: "CREATE TRIGGER log_update AFTER UPDATE ON users BEGIN SELECT 1; END",
    enabled: true,
  },
  {
    name: "validate_insert",
    tableName: "users",
    schema: "public",
    timing: "BEFORE",
    event: "INSERT",
    definition: "CREATE TRIGGER validate_insert BEFORE INSERT ON users BEGIN SELECT 1; END",
    enabled: true,
  },
  {
    name: "audit_delete",
    tableName: "orders",
    schema: "public",
    timing: "AFTER",
    event: "DELETE",
    definition: "CREATE TRIGGER audit_delete AFTER DELETE ON orders BEGIN SELECT 1; END",
    enabled: false,
  },
];

vi.mock("../queries/schema.queries", () => ({
  useExecuteDdl: () => ({
    mutate: vi.fn(),
    isPending: false,
  }),
  useIntrospect: () => ({
    data: { triggers: mockTriggers },
  }),
}));

describe("TriggerManager", () => {
  it("shows triggers for the current table only", () => {
    render(<TriggerManager connectionId="conn-1" schema="public" table="users" />);

    // Should show the two triggers for "users" table.
    expect(screen.getByText("log_update")).toBeTruthy();
    expect(screen.getByText("validate_insert")).toBeTruthy();

    // Should NOT show the trigger for "orders" table.
    expect(screen.queryByText("audit_delete")).toBeNull();
  });

  it("displays timing and event badges", () => {
    render(<TriggerManager connectionId="conn-1" schema="public" table="users" />);

    // AFTER and BEFORE appear both as badges and as Select options.
    expect(screen.getAllByText("AFTER").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("BEFORE").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("UPDATE").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("INSERT").length).toBeGreaterThanOrEqual(1);
  });

  it("shows trigger count in header", () => {
    render(<TriggerManager connectionId="conn-1" schema="public" table="users" />);

    // Header should show "schema.triggers (2)" for the 2 triggers on "users".
    expect(screen.getByText("schema.triggers (2)")).toBeTruthy();
  });

  it("shows no-triggers message for table without triggers", () => {
    render(<TriggerManager connectionId="conn-1" schema="public" table="empty_table" />);

    expect(screen.getByText("schema.noTriggers")).toBeTruthy();
  });

  it("shows DISABLED badge for disabled triggers", () => {
    render(<TriggerManager connectionId="conn-1" schema="public" table="orders" />);

    expect(screen.getByText("audit_delete")).toBeTruthy();
    expect(screen.getByText("DISABLED")).toBeTruthy();
  });

  it("renders CREATE trigger form", () => {
    render(<TriggerManager connectionId="conn-1" schema="public" table="users" />);

    // The form section header
    const createHeaders = screen.getAllByText("schema.createTrigger");
    expect(createHeaders.length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("schema.triggerName")).toBeTruthy();
    expect(screen.getByText("schema.triggerTiming")).toBeTruthy();
    expect(screen.getByText("schema.triggerEvent")).toBeTruthy();
    expect(screen.getByText("schema.triggerBody")).toBeTruthy();
  });

  it("renders enable/disable toggle for PostgreSQL connections", () => {
    render(<TriggerManager connectionId="conn-1" schema="public" table="users" />);

    // Both triggers on "users" are enabled, so should show "Disable" button
    const disableButtons = screen.getAllByText("schema.disableTrigger");
    expect(disableButtons.length).toBe(2);
  });
});
