import { Component, type ErrorInfo, type ReactNode } from "react";

import { Button } from "@/components/ui/button";

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

class ErrorBoundaryInner extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
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
          className="text-foreground"
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            padding: "var(32px, 32px)",
            gap: "var(16px, 16px)",
            textAlign: "center",
          }}
        >
          <div
            className="text-destructive"
            style={{
              fontSize: "var(20px, 20px)",
              fontWeight: 600,
            }}
          >
            {this.props.moduleName} — Error
          </div>
          <div className="text-[var(--app-text-muted)]">{this.state.error.message}</div>
          <Button type="button" variant="outline" className="bg-background" onClick={this.reset}>
            Retry
          </Button>
        </div>
      );
    }

    return this.props.children;
  }
}

export function ErrorBoundaryFallback({ error, reset }: { error: Error; reset: () => void }) {
  const { t } = useTranslation();

  return (
    <div
      className="text-foreground"
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        padding: "var(32px, 32px)",
        gap: "var(16px, 16px)",
        textAlign: "center",
      }}
    >
      <div
        className="text-destructive"
        style={{
          fontSize: "var(20px, 20px)",
          fontWeight: 600,
        }}
      >
        {t("common.states.error")}
      </div>
      <div className="text-[var(--app-text-muted)]">{error.message}</div>
      <Button type="button" variant="outline" className="h-9 px-4 bg-background" onClick={reset}>
        {t("common.actions.retry")}
      </Button>
    </div>
  );
}

export const ErrorBoundary = ErrorBoundaryInner;
