import { useMemo, useState, type FormEvent } from "react";
import { Eye, EyeOff, KeyRound } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { DbEngine } from "@/types";

import type { ConnectionDraft } from "./draft";
import { patchDraftForEngine } from "./draft";

type ConnectionFormProps = {
  mode: "create" | "edit";
  draft: ConnectionDraft;
  saving: boolean;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  onDraftChange: (nextDraft: ConnectionDraft) => void;
  onCancel?: () => void;
  onResetDraft?: () => void;
};

export function ConnectionForm({
  mode,
  draft,
  saving,
  onSubmit,
  onDraftChange,
  onCancel,
  onResetDraft,
}: ConnectionFormProps) {
  const [showPassword, setShowPassword] = useState(false);

  const submitLabel = useMemo(() => {
    if (saving) {
      return mode === "create" ? "Saving..." : "Updating...";
    }
    return mode === "create" ? "Save Connection" : "Update Connection";
  }, [mode, saving]);

  function updateDraft<K extends keyof ConnectionDraft>(
    key: K,
    value: ConnectionDraft[K],
  ) {
    onDraftChange({
      ...draft,
      [key]: value,
    });
  }

  return (
    <form className="space-y-4" onSubmit={onSubmit}>
      <div className="space-y-2">
        <Label htmlFor="name">Name</Label>
        <Input
          id="name"
          value={draft.name}
          onChange={(event) => updateDraft("name", event.target.value)}
          placeholder="Analytics Primary"
          required
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="engine">Engine</Label>
        <div id="engine" className="grid grid-cols-3 gap-1 rounded-md border p-1">
          {(["sqlite", "postgres", "mysql"] as const).map((engine) => (
            <Button
              key={engine}
              type="button"
              variant={draft.engine === engine ? "default" : "ghost"}
              size="sm"
              className="h-8"
              onClick={() =>
                onDraftChange(patchDraftForEngine(draft, engine as DbEngine))
              }
            >
              {engine === "sqlite"
                ? "SQLite"
                : engine === "postgres"
                  ? "Postgres"
                  : "MySQL"}
            </Button>
          ))}
        </div>
      </div>

      {draft.engine === "sqlite" ? (
        <div className="space-y-2">
          <Label htmlFor="path">SQLite path</Label>
          <Input
            id="path"
            value={draft.path}
            onChange={(event) => updateDraft("path", event.target.value)}
            placeholder="./db-pro.sqlite"
            required
          />
          <p className="text-[11px] text-muted-foreground">
            Relative path will be resolved into app data directory.
          </p>
        </div>
      ) : (
        <>
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-2">
              <Label htmlFor="host">Host</Label>
              <Input
                id="host"
                value={draft.host}
                onChange={(event) => updateDraft("host", event.target.value)}
                placeholder="localhost"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="port">Port</Label>
              <Input
                id="port"
                value={draft.port}
                onChange={(event) => updateDraft("port", event.target.value)}
                placeholder={draft.engine === "mysql" ? "3306" : "5432"}
                required
              />
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="database">Database</Label>
            <Input
              id="database"
              value={draft.database}
              onChange={(event) => updateDraft("database", event.target.value)}
              placeholder="postgres"
              required
            />
          </div>

          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-2">
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                value={draft.username}
                onChange={(event) => updateDraft("username", event.target.value)}
                placeholder="postgres"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password" className="inline-flex items-center gap-1">
                <KeyRound className="h-3.5 w-3.5" />
                Password
              </Label>
              <div className="relative">
                <Input
                  id="password"
                  type={showPassword ? "text" : "password"}
                  value={draft.password}
                  onChange={(event) => updateDraft("password", event.target.value)}
                  placeholder={mode === "edit" ? "Enter new password" : "••••••"}
                  className="pr-10"
                  required={mode === "create"}
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  className="absolute right-1 top-1 h-7 w-7"
                  onClick={() => setShowPassword((previous) => !previous)}
                >
                  {showPassword ? (
                    <EyeOff className="h-3.5 w-3.5" />
                  ) : (
                    <Eye className="h-3.5 w-3.5" />
                  )}
                  <span className="sr-only">Toggle password visibility</span>
                </Button>
              </div>
              {mode === "edit" && (
                <p className="text-[11px] text-muted-foreground">
                  Leave empty to keep existing password in keychain.
                </p>
              )}
            </div>
          </div>
        </>
      )}

      <div className="flex items-center justify-end gap-2 pt-1">
        {onResetDraft && (
          <Button
            type="button"
            variant="ghost"
            onClick={onResetDraft}
            disabled={saving}
          >
            Reset Fields
          </Button>
        )}
        {onCancel && (
          <Button
            type="button"
            variant="outline"
            onClick={onCancel}
            disabled={saving}
          >
            Cancel
          </Button>
        )}
        <Button type="submit" disabled={saving}>
          {submitLabel}
        </Button>
      </div>
    </form>
  );
}
