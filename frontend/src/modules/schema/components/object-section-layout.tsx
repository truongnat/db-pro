import type { ReactNode } from "react";

interface ObjectSectionLayoutProps {
  toolbar?: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
}

export function ObjectSectionLayout({ toolbar, footer, children }: ObjectSectionLayoutProps) {
  return (
    <div className="grid h-full min-h-0 grid-rows-[auto_minmax(0,1fr)_auto]">
      {toolbar}
      <div className="min-h-0 overflow-auto">{children}</div>
      {footer}
    </div>
  );
}
