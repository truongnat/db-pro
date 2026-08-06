import { createRootRoute, Outlet } from "@tanstack/react-router";

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
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
          <div className="p-4">
            <p style={{ color: "var(--color-text-secondary)" }}>Navigation</p>
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
