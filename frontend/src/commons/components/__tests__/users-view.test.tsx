import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useConnectionStore } from "@/commons/stores/connection.store";
import * as connectionQueries from "@/modules/connection/queries/connection.queries";
import { UsersView } from "../shell/sidebar-views/users-view";

vi.mock("@/commons/locales/useTranslation", () => ({
  useTranslation: () => ({
    t: (key: string) =>
      ({
        "userManagement.title": "User Management",
        "userManagement.connectFirst": "Connect to a database to manage users",
        "userManagement.postgresOnly": "User management is PostgreSQL-only",
        "userManagement.postgresOnlyReason":
          "SQLite has no database roles or server-level privileges to manage.",
        "userManagement.sidebarHint": "Select a connection to manage users and roles",
      })[key] ?? key,
  }),
}));

vi.mock("@/modules/connection/queries/connection.queries", () => ({
  useConnectionList: vi.fn(),
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

    expect(screen.getByText("Select a connection to manage users and roles")).toBeInTheDocument();
    expect(screen.queryByText("User management is PostgreSQL-only")).not.toBeInTheDocument();
  });
});
