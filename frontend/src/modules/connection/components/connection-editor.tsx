import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import { useTranslation } from "@/commons/locales/useTranslation";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

import type { ConnectionFormData, DriverType, SshTunnelConfig, SslMode } from "../types/connection.types";
import { ColorPicker } from "./color-picker";
import { FormCheckbox } from "./connection-form/form-checkbox";
import { FormInput } from "./connection-form/form-input";
import { FormSelect } from "./connection-form/form-select";
import { TagInput } from "./tag-input";

const DRIVER_OPTIONS = [
  { value: "postgres", label: "PostgreSQL" },
  { value: "sqlite", label: "SQLite" },
];

const SSL_OPTIONS = [
  { value: "disable", label: "Disable" },
  { value: "require", label: "Require" },
  { value: "verify-ca", label: "Verify CA" },
  { value: "verify-full", label: "Verify Full" },
];

const DEFAULT_FORM_DATA: ConnectionFormData = {
  name: "",
  host: "localhost",
  port: 5432,
  database: "",
  username: "",
  driver: "postgres",
  sslMode: "disable",
  queryTimeoutMs: 30000,
  maxRows: 500,
};

interface ConnectionEditorProps {
  initialData?: Partial<ConnectionFormData>;
  isEdit?: boolean;
  onSubmit: (data: ConnectionFormData, password: string, intent: SaveIntent) => void;
  onTest?: (data: ConnectionFormData, password: string) => void;
  onTestSshTunnel?: (config: SshTunnelConfig) => void;
  onCancel: () => void;
  isSubmitting?: boolean;
  isTesting?: boolean;
  isConnecting?: boolean;
  testResult?: "success" | "error" | null;
  connectError?: string | null;
}

export type SaveIntent = "save" | "save-and-connect";

export function ConnectionEditor({
  initialData,
  isEdit = false,
  onSubmit,
  onTest,
  onTestSshTunnel,
  onCancel,
  isSubmitting = false,
  isTesting = false,
  isConnecting = false,
  testResult = null,
  connectError = null,
}: ConnectionEditorProps) {
  const { t } = useTranslation();
  const [formData, setFormData] = useState<ConnectionFormData>({
    ...DEFAULT_FORM_DATA,
    ...initialData,
  });
  const [password, setPassword] = useState("");
  const [showSsh, setShowSsh] = useState(!!initialData?.sshTunnel);

  const isPostgres = formData.driver === "postgres";

  const updateField = <K extends keyof ConnectionFormData>(key: K, value: ConnectionFormData[K]) => {
    setFormData((prev: ConnectionFormData) => ({ ...prev, [key]: value }));
  };

  const updateSshField = (key: string, value: string | number) => {
    setFormData((prev: ConnectionFormData) => ({
      ...prev,
      sshTunnel: { ...prev.sshTunnel!, [key]: value },
    }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const submitter = (e.nativeEvent as SubmitEvent).submitter as HTMLButtonElement | null;
    const intent: SaveIntent = submitter?.name === "save-and-connect" ? "save-and-connect" : "save";
    onSubmit(formData, password, intent);
  };

  const handleTest = () => {
    onTest?.(formData, password);
  };

  const handleDriverChange = (driver: DriverType) => {
    setFormData((prev: ConnectionFormData) => ({
      ...prev,
      driver,
      host: driver === "postgres" ? "localhost" : "",
      port: driver === "postgres" ? 5432 : 0,
      username: driver === "postgres" ? "" : "",
      sslMode: driver === "postgres" ? prev.sslMode : "disable",
    }));
    if (driver === "sqlite") {
      setShowSsh(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-5">
      <FormInput
        label={t("common.labels.name")}
        value={formData.name}
        onChange={(e) => updateField("name", e.target.value)}
        required
        placeholder="My Database"
      />

      <div className="grid grid-cols-2 gap-4">
        <ColorPicker
          value={formData.color}
          onChange={(color) => updateField("color", color)}
        />
        <FormInput
          label={t("connection.group")}
          value={formData.group ?? ""}
          onChange={(e) => updateField("group", e.target.value || undefined)}
          placeholder={t("connection.groupPlaceholder")}
        />
      </div>

      <TagInput
        tags={formData.tags ?? []}
        onChange={(tags) => updateField("tags", tags)}
      />

      <div className="grid grid-cols-2 gap-4">
        <FormSelect
          label={t("common.labels.driver")}
          value={formData.driver}
          onChange={(val) => handleDriverChange(val as DriverType)}
          options={DRIVER_OPTIONS}
          required
        />
        {isPostgres && (
          <FormSelect
            label="SSL Mode"
            value={formData.sslMode}
            onChange={(val) => updateField("sslMode", val as SslMode)}
            options={SSL_OPTIONS}
          />
        )}
      </div>

      {isPostgres ? (
        <div className="grid grid-cols-3 gap-4">
          <FormInput
            label={t("common.labels.host")}
            value={formData.host}
            onChange={(e) => updateField("host", e.target.value)}
            required
            placeholder="localhost"
          />
          <FormInput
            label={t("common.labels.port")}
            type="number"
            value={formData.port}
            onChange={(e) => updateField("port", Number(e.target.value))}
            required
            min={1}
            max={65535}
          />
          <FormInput
            label={t("common.labels.database")}
            value={formData.database}
            onChange={(e) => updateField("database", e.target.value)}
            required
          />
        </div>
      ) : (
        <div className="flex flex-col gap-1.5">
          <label className="text-[12px] font-medium text-foreground">
            {t("connection.filePath")}
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              className="flex-1 rounded-md border border-[var(--app-border)] bg-background px-3 py-1.5 text-[13px] outline-none transition-colors focus:border-primary"
              value={formData.database}
              onChange={(e) => updateField("database", e.target.value)}
              required
              placeholder="/path/to/database.db"
            />
            <Button
              type="button"
              variant="outline"
              className="h-[34px] shrink-0 px-3 text-[13px]"
              onClick={async () => {
                const selected = await open({
                  filters: [{ name: "SQLite", extensions: ["db", "sqlite", "sqlite3"] }, { name: "All Files", extensions: ["*"] }],
                  defaultPath: formData.database || undefined,
                });
                if (selected) {
                  updateField("database", selected);
                }
              }}
            >
              Browse…
            </Button>
          </div>
        </div>
      )}

      {isPostgres && (
        <div className="grid grid-cols-2 gap-4">
          <FormInput
            label={t("common.labels.username")}
            value={formData.username}
            onChange={(e) => updateField("username", e.target.value)}
            required
          />
          <FormInput
            label={t("common.labels.password")}
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required={!isEdit}
            placeholder={isEdit ? "(unchanged)" : ""}
          />
        </div>
      )}

      {!isPostgres && (
        <FormInput
          label={t("common.labels.password")}
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder={isEdit ? "(unchanged)" : ""}
        />
      )}

      {isPostgres && (
        <div className="flex flex-col gap-3">
          <FormCheckbox label="Use SSH Tunnel" checked={showSsh} onChange={setShowSsh} />

          {showSsh && (
            <div className="flex flex-col gap-4 rounded-lg border border-[var(--app-border)] bg-muted p-4">
              <div className="grid grid-cols-2 gap-4">
                <FormInput
                  label="SSH Host"
                  value={formData.sshTunnel?.host ?? ""}
                  onChange={(e) => updateSshField("host", e.target.value)}
                  required
                />
                <FormInput
                  label="SSH Port"
                  type="number"
                  value={formData.sshTunnel?.port ?? 22}
                  onChange={(e) => updateSshField("port", Number(e.target.value))}
                  required
                  min={1}
                  max={65535}
                />
                <FormInput
                  label="SSH User"
                  value={formData.sshTunnel?.user ?? ""}
                  onChange={(e) => updateSshField("user", e.target.value)}
                  required
                />
                <FormInput
                  label="Private Key Path"
                  value={formData.sshTunnel?.privateKeyPath ?? ""}
                  onChange={(e) => updateSshField("privateKeyPath", e.target.value)}
                  required
                  placeholder="~/.ssh/id_rsa"
                />
                <FormInput
                  label="Key Passphrase"
                  type="password"
                  value={formData.sshTunnel?.password ?? ""}
                  onChange={(e) => updateSshField("password", e.target.value)}
                  placeholder="(optional)"
                />
              </div>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={!onTestSshTunnel}
                onClick={() => {
                  if (formData.sshTunnel) {
                    onTestSshTunnel?.(formData.sshTunnel);
                  }
                }}
              >
                {onTestSshTunnel ? "Test Tunnel" : "Test Tunnel (Coming soon)"}
              </Button>
            </div>
          )}
        </div>
      )}

      <div className="grid grid-cols-2 gap-4">
        <FormInput
          label="Query Timeout (ms)"
          type="number"
          value={formData.queryTimeoutMs}
          onChange={(e) => updateField("queryTimeoutMs", Number(e.target.value))}
          min={1000}
          max={300000}
        />
        <FormInput
          label="Max Rows"
          type="number"
          value={formData.maxRows}
          onChange={(e) => updateField("maxRows", Number(e.target.value))}
          min={1}
          max={100000}
        />
      </div>

      <FormCheckbox
        label={t("connection.readonly")}
        checked={formData.readonly ?? false}
        onChange={(checked) => updateField("readonly", checked)}
      />

      {testResult && (
        <div
          className="flex items-center gap-2 rounded-lg border border-[var(--app-border)] bg-muted px-4 py-3"
        >
          <Badge variant={testResult === "success" ? "success" : "error"} dot>
            {testResult === "success" ? t("connection.testSuccess") : t("connection.testFailed")}
          </Badge>
        </div>
      )}

      {connectError && (
        <div className="flex items-center gap-2 rounded-lg border border-destructive/50 bg-destructive/5 px-4 py-3">
          <Badge variant="error" dot>
            {connectError}
          </Badge>
        </div>
      )}

      <div className="flex justify-end gap-2 border-t border-[var(--app-border-subtle)] pt-4">
        <Button type="button" variant="outline" onClick={onCancel}>
          {t("common.actions.cancel")}
        </Button>
        {onTest && (
          <Button
            type="button"
            variant="outline"
            onClick={handleTest}
            loading={isTesting}
          >
            {isTesting ? t("common.states.loading") : t("connection.test")}
          </Button>
        )}
        <Button type="submit" name="save" loading={isSubmitting}>
          {isSubmitting ? t("common.states.loading") : t("common.actions.save")}
        </Button>
        <Button
          type="submit"
          name="save-and-connect"
          variant="outline"
          loading={isConnecting}
        >
          {isConnecting ? t("common.states.loading") : t("connection.saveAndConnect")}
        </Button>
      </div>
    </form>
  );
}
