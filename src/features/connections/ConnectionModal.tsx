import { useEffect, type FormEvent } from "react";
import { X } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

import { ConnectionForm } from "./ConnectionForm";
import type { ConnectionDraft } from "./draft";

type ConnectionModalProps = {
  open: boolean;
  mode: "create" | "edit";
  draft: ConnectionDraft;
  saving: boolean;
  onClose: () => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  onDraftChange: (nextDraft: ConnectionDraft) => void;
  onResetDraft: () => void;
};

export function ConnectionModal({
  open,
  mode,
  draft,
  saving,
  onClose,
  onSubmit,
  onDraftChange,
  onResetDraft,
}: ConnectionModalProps) {
  useEffect(() => {
    if (!open) {
      return;
    }

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", handleEscape);
    return () => {
      window.removeEventListener("keydown", handleEscape);
    };
  }, [onClose, open]);

  if (!open) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/55 p-4 backdrop-blur-sm"
      onClick={onClose}
    >
      <Card
        className="w-full max-w-2xl border-white/60 bg-background/95 shadow-2xl"
        onClick={(event) => event.stopPropagation()}
      >
        <CardHeader className="flex flex-row items-center justify-between space-y-0">
          <div className="space-y-1">
            <CardTitle>{mode === "create" ? "Add Connection" : "Edit Connection"}</CardTitle>
            <CardDescription>
              {mode === "create"
                ? "Create a new connection profile. Passwords are stored in system keychain."
                : "Update connection details and keep credentials secured in keychain."}
            </CardDescription>
          </div>
          <Button type="button" variant="ghost" size="icon" onClick={onClose}>
            <X className="h-4 w-4" />
            <span className="sr-only">Close</span>
          </Button>
        </CardHeader>
        <CardContent>
          <ConnectionForm
            mode={mode}
            draft={draft}
            saving={saving}
            onSubmit={onSubmit}
            onDraftChange={onDraftChange}
            onCancel={onClose}
            onResetDraft={onResetDraft}
          />
        </CardContent>
      </Card>
    </div>
  );
}
