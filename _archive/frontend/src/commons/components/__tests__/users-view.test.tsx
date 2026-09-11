import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useConnectionStore } from "@/commons/stores/connection.store";
import * as connectionQueries from "@/modules/connection/queries/connection.queries";
import { UsersView } from "../shell/sidebar-views/users-view";

const mockUsers = vi.hoisted(() => ({
  list: vi.fn(),
  privileges: vi.fn(),
  drop: vi.fn(),
}));

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { name?: string; count?: number }) => {
      const value =
        {
          "userManagement.title": "User Management",
          "userManagement.connectFirst": "Connect to a database to manage users",
          "userManagement.postgresOnly": "User management is PostgreSQL-only",
          "userManagement.postgresOnlyReason":
            "SQLite has no database roles or server-level privileges to manage.",
          "userManagement.sidebarHint": "Select a connection to manage users and roles",
          "userManagement.roles": "Roles",
          "userManagement.login": "Login",
          "userManagement.noLogin": "No Login",
          "userManagement.dropRole": "Drop Role",
          "userManagement.privilegeCount": "{{count}} table privilege(s)",
          "userManagement.confirmDropImpact":
            'Dropping "{{name}}" will remove the role and its {{count}} table privilege(s). This cannot be undone.',
          "common.states.empty": "No roles found",
          "common.states.loading": "Loading",
          "common.states.error": "Error",
          "common.actions.cancel": "Cancel",
        }[key] ?? key;
      return value
        .replace("{{name}}", options?.name ?? "")
        .replace("{{count}}", String(options?.count ?? 0));
    },
  }),
}));

vi.mock("@/modules/connection/queries/connection.queries", () => ({
  useConnectionList: vi.fn(),
}));

vi.mock("@/modules/user-management/queries/user.queries", () => ({
  useListUsers: () => mockUsers.list(),
  useListPrivileges: () => mockUsers.privileges(),
  useDropRole: () => mockUsers.drop(),
}));

const connections = [
  {
    id: "sqlite-1",
    name: "Local SQLite",
    driver: "sqlite" as const,
  },
  {
    id: "postgres-1",
    name: "Local PostgreSQL",
    driver: "postgres" as const,
  },
];

describe("UsersView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(connectionQueries.useConnectionList).mockReturnValue({
      data: connections,
      isLoading: false,
    } as ReturnType<typeof connectionQueries.useConnectionList>);
    mockUsers.list.mockReturnValue({
      data: [],
      isPending: false,
      isError: false,
    });
    mockUsers.privileges.mockReturnValue({ data: [] });
    mockUsers.drop.mockReturnValue({ isPending: false, mutate: vi.fn() });
    useConnectionStore.setState({ explorerConnectionId: null });
  });

  it("explains when no connection is selected", () => {
    render(<UsersView />);

    expect(screen.getByText("Connect to a database to manage users")).toBeInTheDocument();
  });

  it("shows a PostgreSQL-only capability explanation for SQLite", () => {
    useConnectionStore.setState({ explorerConnectionId: "sqlite-1" });
    render(<UsersView />);

    expect(screen.getByText("User management is PostgreSQL-only")).toBeInTheDocument();
    expect(
      screen.getByText("SQLite has no database roles or server-level privileges to manage."),
    ).toBeInTheDocument();
  });

  it("keeps the connected PostgreSQL view available", () => {
    useConnectionStore.setState({ explorerConnectionId: "postgres-1" });
    render(<UsersView />);

    expect(screen.getByText("No roles found")).toBeInTheDocument();
    expect(screen.queryByText("User management is PostgreSQL-only")).not.toBeInTheDocument();
  });

  it("summarizes privileges before dropping a role", async () => {
    const user = userEvent.setup();
    const mutate = vi.fn();
    mockUsers.list.mockReturnValue({
      data: [{ name: "reporting", canLogin: true }],
      isPending: false,
      isError: false,
    });
    mockUsers.privileges.mockReturnValue({ data: [{ schema: "public", table: "orders" }] });
    mockUsers.drop.mockReturnValue({ isPending: false, mutate });
    useConnectionStore.setState({ explorerConnectionId: "postgres-1" });

    render(<UsersView />);
    await user.click(screen.getByRole("button", { name: "Drop Role" }));

    expect(
      screen.getByText(
        'Dropping "reporting" will remove the role and its 1 table privilege(s). This cannot be undone.',
      ),
    ).toBeInTheDocument();
    await user.click(
      within(screen.getByRole("alertdialog")).getByRole("button", { name: "Drop Role" }),
    );
    expect(mutate).toHaveBeenCalledWith("reporting");
  });
});
