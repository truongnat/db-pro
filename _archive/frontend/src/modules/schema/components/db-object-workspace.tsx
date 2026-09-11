import type { ReactNode } from "react";

import type { DbObjectSection } from "@/commons/types/workspace.types";

import { ObjectSectionTabs } from "./object-section-tabs";

interface DbObjectWorkspaceProps {
  activeSection: DbObjectSection;
  onSelectSection: (section: DbObjectSection) => void;
  children: ReactNode;
}

export function DbObjectWorkspace({
  activeSection,
  onSelectSection,
  children,
}: DbObjectWorkspaceProps) {
  return (
    <div className="grid h-full min-h-0 overflow-hidden grid-rows-[auto_minmax(0,1fr)]">
      <ObjectSectionTabs activeSection={activeSection} onSelect={onSelectSection} />
      <div className="min-h-0 overflow-hidden">{children}</div>
    </div>
  );
}
