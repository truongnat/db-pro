import { Component, type ErrorInfo, type ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

type AppErrorBoundaryProps = {
  children: ReactNode;
};

type AppErrorBoundaryState = {
  hasError: boolean;
  message: string;
};

export class AppErrorBoundary extends Component<
  AppErrorBoundaryProps,
  AppErrorBoundaryState
> {
  state: AppErrorBoundaryState = {
    hasError: false,
    message: "",
  };

  static getDerivedStateFromError(error: Error): AppErrorBoundaryState {
    return {
      hasError: true,
      message: error.message || "Unexpected runtime error",
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error("App crashed", error);
    console.error("Component stack:\n" + errorInfo.componentStack);
  }

  private handleReload = () => {
    window.location.reload();
  };

  render() {
    if (!this.state.hasError) {
      return this.props.children;
    }

    return (
      <div className="flex min-h-screen items-center justify-center bg-background p-6 text-foreground">
        <Card className="w-full max-w-xl border-border/80">
          <CardHeader>
            <CardTitle>Ứng dụng gặp lỗi runtime</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="rounded-md border border-destructive/30 bg-destructive/10 p-3 font-mono text-sm text-destructive">
              {this.state.message}
            </p>
            <p className="text-sm text-muted-foreground">
              Nếu bạn đang chạy web thuần, hãy dùng Tauri: <code>npm run tauri dev</code>.
            </p>
            <Button type="button" onClick={this.handleReload}>
              Reload
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }
}
