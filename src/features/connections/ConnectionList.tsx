import { PencilLine, ShieldCheck, Trash2 } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import type { ConnectionListItem } from "@/types";

type ConnectionListProps = {
  items: ConnectionListItem[];
  selectedConnectionId: string | null;
  busy: boolean;
  onEdit: (connectionId: string) => void;
  onSelect: (connectionId: string) => void;
  onDelete: (connectionId: string) => void;
  isProtected: (connectionId: string) => boolean;
};

export function ConnectionList({
  items,
  selectedConnectionId,
  busy,
  onEdit,
  onSelect,
  onDelete,
  isProtected,
}: ConnectionListProps) {
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <span>Connections</span>
        <Badge variant="secondary">{items.length}</Badge>
      </div>

      <ScrollArea className="min-h-0 flex-1 pr-2">
        <div className="space-y-2">
          {items.length === 0 && (
            <div className="rounded-md border border-dashed bg-muted/50 p-3 text-xs text-muted-foreground">
              No connections yet.
            </div>
          )}

          {items.map((item) => {
            const protectedConnection = isProtected(item.id);

            return (
              <Card
                key={item.id}
                className={cn(
                  "group cursor-pointer border p-2 transition-colors",
                  item.id === selectedConnectionId
                    ? "border-primary/40 bg-primary/5 shadow-sm"
                    : "border-border/70 bg-background/80 hover:bg-accent/60",
                )}
                onClick={() => onSelect(item.id)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    onSelect(item.id);
                  }
                }}
                tabIndex={0}
                role="button"
              >
                <CardContent className="flex items-start justify-between gap-3 p-0">
                  <div className="space-y-1">
                    <p className="text-sm font-medium leading-none">{item.name}</p>
                    <p className="break-all pr-2 text-xs text-muted-foreground">
                      {item.target}
                    </p>
                  </div>

                  <div className="ml-3 flex items-center gap-1">
                    <Badge variant="outline" className="uppercase">
                      {item.engine}
                    </Badge>
                    {protectedConnection && (
                      <ShieldCheck className="h-3.5 w-3.5 text-emerald-600" />
                    )}
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6 opacity-70 transition group-hover:opacity-100"
                      onClick={(event) => {
                        event.stopPropagation();
                        onEdit(item.id);
                      }}
                      disabled={busy}
                    >
                      <PencilLine className="h-3.5 w-3.5" />
                      <span className="sr-only">Edit connection</span>
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6 opacity-70 transition group-hover:opacity-100"
                      onClick={(event) => {
                        event.stopPropagation();
                        onDelete(item.id);
                      }}
                      disabled={busy || protectedConnection}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                      <span className="sr-only">Delete connection</span>
                    </Button>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      </ScrollArea>
    </div>
  );
}
