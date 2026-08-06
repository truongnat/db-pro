import { createRootRoute, Link, Outlet, useLocation } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: RootLayout,
});

const NAV_ITEMS = [
  { to: "/connections", label: "Connections" },
  { to: "/query", label: "Query" },
  { to: "/schema", label: "Schema" },
  { to: "/data", label: "Data" },
  { to: "/users", label: "Users" },
] as const;

function RootLayout() {
  const location = useLocation();

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      <header
        className="flex shrink-0 items-center border-b px-4"
        style={{
          height: "var(--appbar-height)",
          borderColor: "var(--color-border)",
          backgroundColor: "var(--color-surface)",
        }}
      >
        <span className="text-lg font-semibold" style={{ color: "var(--color-text)" }}>
          DB Pro
        </span>
      </header>

      <div className="flex min-h-0 flex-1">
        <nav
          className="shrink-0 border-r"
          style={{
            width: "var(--sidebar-width)",
            borderColor: "var(--color-border)",
            backgroundColor: "var(--color-surface)",
          }}
        >
          <div className="flex flex-col gap-1 p-2">
            {NAV_ITEMS.map((item) => {
              const isActive = location.pathname.startsWith(item.to);
              return (
                <Link
                  key={item.to}
                  to={item.to}
                  className="rounded-[var(--radius-sm)] px-3 py-2 text-sm transition-colors hover:bg-[var(--color-bg)]"
                  style={{
                    color: isActive ? "var(--color-text)" : "var(--color-text-secondary)",
                    backgroundColor: isActive ? "var(--color-bg)" : undefined,
                    fontWeight: isActive ? 500 : 400,
                  }}
                >
                  {item.label}
                </Link>
              );
            })}
          </div>
        </nav>

        <main className="flex-1 overflow-auto p-4">
          <Outlet />
        </main>
      </div>

      <footer
        className="flex shrink-0 items-center border-t px-4"
        style={{
          height: "var(--statusbar-height)",
          borderColor: "var(--color-border)",
          backgroundColor: "var(--color-surface)",
        }}
      >
        <span className="text-xs" style={{ color: "var(--color-text-secondary)" }}>
          Ready
        </span>
      </footer>
    </div>
  );
}
