import * as React from "react";
import {
  CircleCheckIcon,
  InfoIcon,
  TriangleAlertIcon,
  OctagonXIcon,
  Loader2Icon,
} from "lucide-react";
import { Toaster as Sonner, type ToasterProps } from "sonner";

import { useThemeStore } from "@/commons/stores/theme.store";

const Toaster = ({ ...props }: ToasterProps) => {
  const mode = useThemeStore((s) => s.mode);

  return (
    <Sonner
      theme={mode === "system" ? "system" : mode}
      className="toaster group"
      icons={{
        success: <CircleCheckIcon className="size-4" />,
        info: <InfoIcon className="size-4" />,
        warning: <TriangleAlertIcon className="size-4" />,
        error: <OctagonXIcon className="size-4" />,
        loading: <Loader2Icon className="size-4 animate-spin" />,
      }}
      style={
        {
          "--normal-bg": "var(--surface-floating)",
          "--normal-text": "var(--text-primary)",
          "--normal-border": "var(--border-default)",
          "--border-radius": "var(--radius)",
        } as React.CSSProperties
      }
      toastOptions={{
        classNames: {
          toast: "cn-toast",
        },
      }}
      visibleToasts={3}
      closeButton
      {...props}
    />
  );
};

export { Toaster };
