import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { I18nextProvider, initReactI18next } from "react-i18next";
import i18n from "i18next";

import { ConnectionEditor } from "../components/connection-editor";

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        common: {
          labels: {
            name: "Name",
            host: "Host",
            port: "Port",
            database: "Database",
            username: "Username",
            password: "Password",
            driver: "Driver",
          },
          actions: { save: "Save", cancel: "Cancel" },
          states: { loading: "Loading..." },
        },
        connection: {
          test: "Test Connection",
          testSuccess: "Test successful",
          testFailed: "Test failed",
          filePath: "File Path",
          saveAndConnect: "Save & Connect",
          readonly: "Read only",
          group: "Group",
          groupPlaceholder: "Group",
        },
      },
    },
  },
  lng: "en",
  fallbackLng: "en",
});

function renderEditor(props: Partial<React.ComponentProps<typeof ConnectionEditor>> = {}) {
  const defaults = {
    onSubmit: vi.fn(),
    onCancel: vi.fn(),
  };
  return render(
    <I18nextProvider i18n={i18n}>
      <ConnectionEditor {...defaults} {...props} />
    </I18nextProvider>,
  );
}

describe("connection lifecycle P1 regressions", () => {
  it("allows SQLite test and save without a password and strips SSH state", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();
    const onTest = vi.fn();

    renderEditor({
      onSubmit,
      onTest,
      initialData: {
        driver: "sqlite",
        host: "",
        port: 0,
        username: "",
        sslMode: "disable",
        sshTunnel: {
          host: "stale.example.com",
          port: 22,
          user: "stale",
          privateKeyPath: "/tmp/stale",
        },
      },
    });

    await user.type(screen.getByPlaceholderText("My Database"), "SQLite DB");
    await user.type(screen.getByPlaceholderText("/path/to/database.db"), "/tmp/test.db");

    await user.click(screen.getByRole("button", { name: "Test Connection" }));
    expect(onTest).toHaveBeenCalledWith(
      expect.objectContaining({ driver: "sqlite", database: "/tmp/test.db", sshTunnel: undefined }),
      "",
    );

    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({ driver: "sqlite", database: "/tmp/test.db", sshTunnel: undefined }),
      "",
      "save",
    );
  });

  it("defaults SSH port to 22 while preserving an omitted passphrase as undefined", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();

    renderEditor({ onSubmit });

    await user.type(screen.getByPlaceholderText("My Database"), "Postgres SSH");
    await user.type(screen.getByLabelText(/database/i), "app");
    await user.type(screen.getByLabelText(/username/i), "postgres");
    await user.type(screen.getByLabelText(/password/i), "secret");

    await user.click(screen.getByText("Use SSH Tunnel"));
    await user.type(screen.getByLabelText(/SSH Host/i), "ssh.example.com");
    await user.type(screen.getByLabelText(/SSH User/i), "deploy");
    await user.type(screen.getByLabelText(/Private Key Path/i), "/home/deploy/.ssh/id_ed25519");

    await user.click(screen.getByRole("button", { name: "Save" }));

    expect(onSubmit).toHaveBeenLastCalledWith(
      expect.objectContaining({
        sshTunnel: {
          host: "ssh.example.com",
          port: 22,
          user: "deploy",
          privateKeyPath: "/home/deploy/.ssh/id_ed25519",
          password: undefined,
        },
      }),
      "secret",
      "save",
    );

    await user.click(screen.getByText("Use SSH Tunnel"));
    await user.click(screen.getByRole("button", { name: "Save" }));
    expect(onSubmit).toHaveBeenLastCalledWith(
      expect.objectContaining({ sshTunnel: undefined }),
      "secret",
      "save",
    );
  });
});
