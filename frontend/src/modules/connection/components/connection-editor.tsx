import { useState } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

import type { ConnectionFormData, DriverType, SslMode } from "../types/connection.types";
import { FormCheckbox } from "./connection-form/form-checkbox";
import { FormInput } from "./connection-form/form-input";
import { FormSelect } from "./connection-form/form-select";

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
  onSubmit: (data: ConnectionFormData, password: string) => void;
  onTest?: (data: ConnectionFormData, password: string) => void;
  onCancel: () => void;
  isSubmitting?: boolean;
  isTesting?: boolean;
  testResult?: "success" | "error" | null;
}

export function ConnectionEditor({
  initialData,
  isEdit = false,
  onSubmit,
  onTest,
  onCancel,
  isSubmitting = false,
  isTesting = false,
  testResult = null,
}: ConnectionEditorProps) {
  const { t } = useTranslation();
  const [formData, setFormData] = useState<ConnectionFormData>({
    ...DEFAULT_FORM_DATA,
    ...initialData,
  });
  const [password, setPassword] = useState("");
  const [showSsh, setShowSsh] = useState(!!initialData?.sshTunnel);

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
    onSubmit(formData, password);
  };

  const handleTest = () => {
    onTest?.(formData, password);
  };

  const handleDriverChange = (driver: DriverType) => {
    setFormData((prev: ConnectionFormData) => ({
      ...prev,
      driver,
      port: driver === "postgres" ? 5432 : 0,
    }));
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
        <FormSelect
          label="Driver"
          value={formData.driver}
          onChange={(e) => handleDriverChange(e.target.value as DriverType)}
          options={DRIVER_OPTIONS}
          required
        />
        <FormSelect
          label="SSL Mode"
          value={formData.sslMode}
          onChange={(e) => updateField("sslMode", e.target.value as SslMode)}
          options={SSL_OPTIONS}
        />
      </div>

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

      <div className="flex flex-col gap-3">
        <FormCheckbox label="Use SSH Tunnel" checked={showSsh} onChange={setShowSsh} />

        {showSsh && (
          <div className="grid grid-cols-2 gap-4 rounded-[var(--radius-sm)] border p-4" style={{ borderColor: "var(--color-border)" }}>
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
          </div>
        )}
      </div>

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

      {testResult && (
        <div
          className="rounded-[var(--radius-sm)] px-3 py-2 text-sm"
          style={{
            backgroundColor: testResult === "success" ? "var(--color-success,#22c55e)" : "var(--color-error,#ef4444)",
            color: "white",
          }}
        >
          {testResult === "success" ? t("connection.testSuccess") : t("connection.testFailed")}
        </div>
      )}

      <div className="flex justify-end gap-2 border-t pt-4" style={{ borderColor: "var(--color-border)" }}>
        <button
          type="button"
          onClick={onCancel}
          className="rounded-[var(--radius-sm)] border px-4 py-2 text-sm transition-colors hover:bg-[var(--color-surface)]"
          style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
        >
          {t("common.actions.cancel")}
        </button>
        {onTest && (
          <button
            type="button"
            onClick={handleTest}
            disabled={isTesting}
            className="rounded-[var(--radius-sm)] border px-4 py-2 text-sm transition-colors hover:bg-[var(--color-surface)]"
            style={{ borderColor: "var(--color-border)", color: "var(--color-text)" }}
          >
            {isTesting ? t("common.states.loading") : t("connection.test")}
          </button>
        )}
        <button
          type="submit"
          disabled={isSubmitting}
          className="rounded-[var(--radius-sm)] px-4 py-2 text-sm text-white transition-colors disabled:opacity-50"
          style={{ backgroundColor: "var(--color-primary,#3b82f6)" }}
        >
          {isSubmitting ? t("common.states.loading") : t("common.actions.save")}
        </button>
      </div>
    </form>
  );
}
