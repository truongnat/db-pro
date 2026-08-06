import { createRootRoute, Link, Outlet, useLocation } from "@tanstack/react-router";

import { cn } from "@/lib/utils";

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
      <header className="flex h-12 shrink-0 items-center border-b border-border bg-card px-4">
        <span className="text-lg font-semibold text-foreground">
          DB Pro
        </span>
      </header>

      <div className="flex min-h-0 flex-1">
        <nav className="w-60 shrink-0 border-r border-border bg-card">
          <div className="flex flex-col gap-1 p-2">
            {NAV_ITEMS.map((item) => {
              const isActive = location.pathname.startsWith(item.to);
              return (
                <Link
                  key={item.to}
                  to={item.to}
                  className={cn(
                    "rounded-sm px-3 py-2 text-sm transition-colors hover:bg-background",
                    isActive ? "text-foreground bg-background" : "text-muted-foreground",
                  )}
                  style={{ fontWeight: isActive ? 500 : 400 }}
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

      <footer className="flex h-7 shrink-0 items-center border-t border-border bg-card px-4">
        <span className="text-xs text-muted-foreground">
          Ready
        </span>
      </footer>
    </div>
  );
}
