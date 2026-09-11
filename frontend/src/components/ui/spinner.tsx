import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { Loader2 } from "lucide-react";

import { cn } from "@/lib/utils";

const spinnerVariants = cva(
  "inline-flex shrink-0 items-center justify-center motion-reduce:animate-none",
  {
    variants: {
      size: {
        xs: "size-3",
        sm: "size-3.5",
        default: "size-4",
        lg: "size-5",
        xl: "size-6",
      },
      variant: {
        default: "text-muted-foreground",
        accent: "text-accent",
        primary: "text-primary",
        current: "text-current",
      },
    },
    defaultVariants: {
      size: "default",
      variant: "default",
    },
  },
);

function Spinner({
  className,
  size,
  variant,
  ...props
}: React.ComponentProps<"span"> & VariantProps<typeof spinnerVariants>) {
  return (
    <span
      data-slot="spinner"
      role="status"
      aria-label="Loading"
      className={cn(spinnerVariants({ size, variant }), className)}
      {...props}
    >
      <Loader2 className="size-full animate-spin motion-reduce:animate-none" aria-hidden="true" />
    </span>
  );
}

export { Spinner, spinnerVariants };
