import { describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
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

describe("ConnectionEditor", () => {
  it("renders all form fields for postgres", () => {
    renderEditor();
    expect(screen.getByPlaceholderText("My Database")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("localhost")).toBeInTheDocument();
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Driver")).toBeInTheDocument();
    expect(screen.getByText("Host")).toBeInTheDocument();
    expect(screen.getByText("Port")).toBeInTheDocument();
    expect(screen.getByText("Database")).toBeInTheDocument();
    expect(screen.getByText("Username")).toBeInTheDocument();
    expect(screen.getByText("Password")).toBeInTheDocument();
    expect(screen.getByText("SSL Mode")).toBeInTheDocument();
  });

  it("renders with default values", () => {
    renderEditor();
    const portInput = screen.getByDisplayValue("5432");
    expect(portInput).toBeInTheDocument();
    expect(screen.getByPlaceholderText("localhost")).toHaveValue("localhost");
  });

  it("renders with initial data for editing", () => {
    renderEditor({
      isEdit: true,
      initialData: {
        name: "My DB",
        host: "db.example.com",
        port: 3306,
        database: "production",
        username: "admin",
        driver: "postgres",
        sslMode: "require",
        queryTimeoutMs: 30000,
        maxRows: 500,
      },
    });

    expect(screen.getByDisplayValue("My DB")).toBeInTheDocument();
    expect(screen.getByDisplayValue("db.example.com")).toBeInTheDocument();
    expect(screen.getByDisplayValue("production")).toBeInTheDocument();
  });

  it("calls onSubmit with form data and password", async () => {
    const onSubmit = vi.fn();
    const user = userEvent.setup();

    renderEditor({ onSubmit });

    const nameInput = screen.getByPlaceholderText("My Database");
    await user.clear(nameInput);
    await user.type(nameInput, "Test DB");

    const passwordInputs = screen.getAllByDisplayValue("");
    const passwordInput = passwordInputs.find((el) => el.getAttribute("type") === "password");
    if (passwordInput) await user.type(passwordInput, "secret123");

    await user.click(screen.getByText("Save"));

    expect(onSubmit).toHaveBeenCalledTimes(1);
    const [formData, password] = onSubmit.mock.calls[0];
    expect(password).toBe("secret123");
    expect(formData.name).toBe("Test DB");
  });

  it("calls onCancel when cancel is clicked", async () => {
    const onCancel = vi.fn();
    const user = userEvent.setup();

    renderEditor({ onCancel });
    await user.click(screen.getByText("Cancel"));

    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("shows test button when onTest is provided", () => {
    renderEditor({ onTest: vi.fn() });
    expect(screen.getByText("Test Connection")).toBeInTheDocument();
  });

  it("hides test button when onTest is not provided", () => {
    renderEditor();
    expect(screen.queryByText("Test Connection")).not.toBeInTheDocument();
  });

  it("shows test success result", () => {
    renderEditor({ onTest: vi.fn(), testResult: "success" });
    expect(screen.getByText("Test successful")).toBeInTheDocument();
  });

  it("shows test error result", () => {
    renderEditor({ onTest: vi.fn(), testResult: "error" });
    expect(screen.getByText("Test failed")).toBeInTheDocument();
  });

  it("disables submit button when submitting", () => {
    renderEditor({ isSubmitting: true });
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });

  it("toggles SSH tunnel fields", async () => {
    const user = userEvent.setup();
    renderEditor();

    expect(screen.queryByText("SSH Host")).not.toBeInTheDocument();

    await user.click(screen.getByText("Use SSH Tunnel"));
    expect(screen.getByText("SSH Host")).toBeInTheDocument();
    expect(screen.getByText("SSH Port")).toBeInTheDocument();
    expect(screen.getByText("SSH User")).toBeInTheDocument();
    expect(screen.getByText("Private Key Path")).toBeInTheDocument();
  });

  it("shows file path field for SQLite and hides network fields", async () => {
    const user = userEvent.setup();
    renderEditor();

    const driverTrigger = screen.getAllByRole("combobox")[0];
    await user.click(driverTrigger);

    const sqliteOptions = await screen.findAllByText("SQLite", undefined, { timeout: 2000 });
    const sqliteSpan = sqliteOptions.find((el) => el.tagName === "SPAN");
    await user.click(sqliteSpan!);

    await waitFor(() => {
      expect(screen.getByText("File Path")).toBeInTheDocument();
    });
    expect(screen.queryByText("Host")).not.toBeInTheDocument();
    expect(screen.queryByText("Port")).not.toBeInTheDocument();
    expect(screen.queryByText("Username")).not.toBeInTheDocument();
    expect(screen.queryByText("SSL Mode")).not.toBeInTheDocument();
    expect(screen.queryByText("Use SSH Tunnel")).not.toBeInTheDocument();
  });
});
