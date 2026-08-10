import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface ObjectSectionLayoutProps {
  toolbar?: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
}

export function ObjectSectionLayout({ toolbar, footer, children }: ObjectSectionLayoutProps) {
  const rows =
    toolbar && footer
      ? "grid-rows-[auto_minmax(0,1fr)_auto]"
      : toolbar
        ? "grid-rows-[auto_minmax(0,1fr)]"
        : footer
          ? "grid-rows-[minmax(0,1fr)_auto]"
          : "grid-rows-[minmax(0,1fr)]";
  return (
    <div className={cn("grid h-full min-h-0 overflow-hidden", rows)}>
      {toolbar}
      <div className="flex min-h-0 flex-col overflow-auto">{children}</div>
      {footer}
    </div>
  );
}
