import { Component, type ErrorInfo, type ReactNode } from "react";

interface RootErrorBoundaryProps {
  children: ReactNode;
  onReload?: () => void;
}

interface RootErrorBoundaryState {
  failed: boolean;
}

const FALLBACK_STYLE = {
  minHeight: "100vh",
  display: "grid",
  placeItems: "center",
  background: "#0f1117",
  color: "#f8fafc",
  padding: "24px",
  fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, sans-serif",
} as const;

const CARD_STYLE = {
  width: "min(480px, 100%)",
  border: "1px solid #334155",
  borderRadius: "12px",
  background: "#161822",
  padding: "24px",
  boxSizing: "border-box",
} as const;

const BUTTON_STYLE = {
  marginTop: "20px",
  border: "1px solid #475569",
  borderRadius: "8px",
  background: "#f8fafc",
  color: "#0f172a",
  padding: "10px 14px",
  font: "inherit",
  fontWeight: 600,
  cursor: "pointer",
} as const;

/**
 * Last-resort containment for render/lifecycle failures below the app root.
 *
 * The fallback deliberately avoids the normal design system, application stores,
 * connection state, and error details so it remains available during broad
 * frontend failures without exposing sensitive payloads or clearing recovery data.
 */
export class RootErrorBoundary extends Component<RootErrorBoundaryProps, RootErrorBoundaryState> {
  state: RootErrorBoundaryState = { failed: false };

  static getDerivedStateFromError(): RootErrorBoundaryState {
    return { failed: true };
  }

  componentDidCatch(_error: unknown, _info: ErrorInfo): void {
    // Never log the raw error here. Render errors can contain query text,
    // connection strings, provider payloads, or other sensitive values.
    console.error("Unhandled React error captured by the root error boundary");
  }

  private readonly reload = (): void => {
    if (this.props.onReload) {
      this.props.onReload();
      return;
    }

    window.location.reload();
  };

  render(): ReactNode {
    if (!this.state.failed) {
      return this.props.children;
    }

    return (
      <main style={FALLBACK_STYLE}>
        <section role="alert" aria-live="assertive" style={CARD_STYLE}>
          <h1 style={{ margin: 0, fontSize: "20px", lineHeight: 1.3 }}>
            The application hit an unexpected error
          </h1>
          <p style={{ margin: "12px 0 0", lineHeight: 1.6, color: "#cbd5e1" }}>
            Your saved and crash-recovery data has not been cleared. Reload the interface to
            continue.
          </p>
          <button type="button" onClick={this.reload} style={BUTTON_STYLE}>
            Reload application
          </button>
        </section>
      </main>
    );
  }
}
