import { Component, type ErrorInfo, type ReactNode } from "react";

import { useTranslation } from "@/commons/locales/useTranslation";

interface ErrorBoundaryProps {
  children: ReactNode;
  moduleName: string;
  fallback?: ReactNode | ((error: Error, reset: () => void) => ReactNode);
  onError?: (error: Error, info: ErrorInfo) => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

class ErrorBoundaryInner extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error(
      `[ErrorBoundary] ${this.props.moduleName}: ${error.message}`,
      info.componentStack,
    );
    this.props.onError?.(error, info);
  }

  reset = (): void => {
    this.setState({ hasError: false, error: null });
  };

  render(): ReactNode {
    if (this.state.hasError && this.state.error) {
      if (this.props.fallback) {
        if (typeof this.props.fallback === "function") {
          return this.props.fallback(this.state.error, this.reset);
        }
        return this.props.fallback;
      }

      return (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            padding: "var(--space-8, 32px)",
            gap: "var(--space-4, 16px)",
            color: "var(--color-text)",
            textAlign: "center",
          }}
        >
          <div
            style={{
              fontSize: "var(--text-xl, 20px)",
              fontWeight: 600,
              color: "var(--color-error, #ef4444)",
            }}
          >
            {this.props.moduleName} — Error
          </div>
          <div style={{ color: "var(--color-text-secondary, #9ca3af)" }}>
            {this.state.error.message}
          </div>
          <button
            type="button"
            onClick={this.reset}
            style={{
              height: "var(--button-height, 36px)",
              padding: "0 var(--space-4, 16px)",
              borderRadius: "var(--radius-md, 6px)",
              border: "1px solid var(--color-border)",
              background: "var(--color-surface)",
              color: "var(--color-text)",
              cursor: "pointer",
            }}
          >
            Retry
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

export function ErrorBoundaryFallback({
  error,
  reset,
}: {
  error: Error;
  reset: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        padding: "var(--space-8, 32px)",
        gap: "var(--space-4, 16px)",
        color: "var(--color-text)",
        textAlign: "center",
      }}
    >
      <div
        style={{
          fontSize: "var(--text-xl, 20px)",
          fontWeight: 600,
          color: "var(--color-error, #ef4444)",
        }}
      >
        {t("common.states.error")}
      </div>
      <div style={{ color: "var(--color-text-secondary, #9ca3af)" }}>
        {error.message}
      </div>
      <button
        type="button"
        onClick={reset}
        className="h-[var(--button-height)] px-4 rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text)] cursor-pointer"
      >
        {t("common.actions.retry")}
      </button>
    </div>
  );
}

export const ErrorBoundary = ErrorBoundaryInner;
