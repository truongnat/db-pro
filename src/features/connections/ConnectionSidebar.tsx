import { Database, Plus, RotateCcw } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import type { ConnectionListItem, NavigatorTree } from "@/types";
import type { NavigatorActionRequest } from "@/features/navigator/actions";

import { ConnectionList } from "./ConnectionList";
import { SAMPLE_CONNECTION_ID } from "./constants";
import { SchemaNavigator } from "../navigator/SchemaNavigator";

type ConnectionSidebarProps = {
  busy: boolean;
  resettingData: boolean;
  connections: ConnectionListItem[];
  selectedConnectionId: string | null;
  selectedConnection: ConnectionListItem | null;
  navigatorTree: NavigatorTree | null;
  navigatorLoading: boolean;
  navigatorError: string | null;
  navigatorObjectCount: number;
  onOpenCreateModal: () => void;
  onOpenEditModal: (connectionId: string) => void;
  onResetData: () => void;
  onSelectConnection: (connectionId: string) => void;
  onDeleteConnection: (connectionId: string) => void;
  onRefreshNavigator: () => void;
  onNavigatorAction: (request: NavigatorActionRequest) => void;
};

export function ConnectionSidebar({
  busy,
  resettingData,
  connections,
  selectedConnectionId,
  selectedConnection,
  navigatorTree,
  navigatorLoading,
  navigatorError,
  navigatorObjectCount,
  onOpenCreateModal,
  onOpenEditModal,
  onResetData,
  onSelectConnection,
  onDeleteConnection,
  onRefreshNavigator,
  onNavigatorAction,
}: ConnectionSidebarProps) {
  return (
    <Card className="flex min-h-0 flex-col border-white/70 bg-card/85 shadow-xl backdrop-blur-xl">
      <CardHeader className="space-y-4 pb-4">
        <div className="flex items-start justify-between gap-3">
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Database className="h-4 w-4 text-slate-500" />
              <CardTitle className="text-base">DB Pro</CardTitle>
            </div>
            <CardDescription>Connection manager with persisted profiles.</CardDescription>
          </div>

          <Button type="button" variant="secondary" size="sm" onClick={onOpenCreateModal}>
            <Plus className="mr-1 h-3.5 w-3.5" />
            Add
          </Button>
        </div>

        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={onResetData}
          disabled={busy || resettingData}
        >
          <RotateCcw className="mr-1.5 h-3.5 w-3.5" />
          {resettingData ? "Resetting..." : "Reset Data"}
        </Button>
      </CardHeader>

      <CardContent className="flex min-h-0 flex-1 flex-col gap-4">
        <ConnectionList
          items={connections}
          selectedConnectionId={selectedConnectionId}
          busy={busy}
          onEdit={onOpenEditModal}
          onSelect={onSelectConnection}
          onDelete={onDeleteConnection}
          isProtected={(connectionId) => connectionId === SAMPLE_CONNECTION_ID}
        />

        <Separator />

        <SchemaNavigator
          selectedConnection={!!selectedConnection}
          tree={navigatorTree}
          loading={navigatorLoading}
          error={navigatorError}
          objectCount={navigatorObjectCount}
          onRefresh={onRefreshNavigator}
          onAction={onNavigatorAction}
        />
      </CardContent>
    </Card>
  );
}
