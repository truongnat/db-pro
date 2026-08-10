import { useEffect, useRef, useState } from "react";
import { CheckCircle2, AlertTriangle, XCircle, X } from "lucide-react";

import { Button } from "@/components/ui/button";

export interface TransactionResult {
  kind: "success" | "partial" | "failure";
  succeeded: number;
  failed: number;
  durationMs: number;
}

interface TransactionFeedbackProps {
  result: TransactionResult | null;
  onDismiss: () => void;
  /** Auto-dismiss after ms. Default 5000. */
  autoDismissMs?: number;
}

export function TransactionFeedback({
  result,
  onDismiss,
  autoDismissMs = 5000,
}: TransactionFeedbackProps) {
  const [visible, setVisible] = useState(false);
  const onDismissRef = useRef(onDismiss);
  onDismissRef.current = onDismiss;

  useEffect(() => {
    if (result) {
      setVisible(true);
      const timer = setTimeout(() => {
        setVisible(false);
        onDismissRef.current();
      }, autoDismissMs);
      return () => clearTimeout(timer);
    }
    setVisible(false);
  }, [result, autoDismissMs]);

  if (!result || !visible) return null;

  const { kind, succeeded, failed, durationMs } = result;
  const duration = durationMs < 1000 ? `${durationMs} ms` : `${(durationMs / 1000).toFixed(1)} s`;

  const config = {
    success: {
      icon: CheckCircle2,
      bg: "bg-emerald-500/10",
      border: "border-emerald-500/20",
      text: "text-emerald-600 dark:text-emerald-400",
      iconColor: "text-emerald-500",
    },
    partial: {
      icon: AlertTriangle,
      bg: "bg-amber-500/10",
      border: "border-amber-500/20",
      text: "text-amber-600 dark:text-amber-400",
      iconColor: "text-amber-500",
    },
    failure: {
      icon: XCircle,
      bg: "bg-destructive/10",
      border: "border-destructive/20",
      text: "text-destructive",
      iconColor: "text-destructive",
    },
  }[kind];

  const Icon = config.icon;

  let message: string;
  if (kind === "success") {
    message = `${succeeded} change${succeeded !== 1 ? "s" : ""} applied`;
  } else if (kind === "partial") {
    message = `${succeeded} succeeded, ${failed} failed`;
  } else {
    message = "Apply failed";
  }

  return (
    <div
      className={`flex items-center gap-2 border-b ${config.border} ${config.bg} px-3 py-1.5 text-xs`}
    >
      <Icon className={`h-3.5 w-3.5 shrink-0 ${config.iconColor}`} />
      <span className={`font-medium ${config.text}`}>{message}</span>
      <span className="text-[var(--app-text-muted)]">{duration}</span>
      <div className="flex-1" />
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="h-5 w-5 p-0 text-[var(--app-text-muted)] hover:text-foreground"
        onClick={() => {
          setVisible(false);
          onDismiss();
        }}
      >
        <X className="h-3 w-3" />
      </Button>
    </div>
  );
}
