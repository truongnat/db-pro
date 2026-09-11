import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { Toggle as TogglePrimitive } from "radix-ui";

import { cn } from "@/lib/utils";

const toggleVariants = cva(
  "group/toggle inline-flex shrink-0 cursor-pointer items-center justify-center gap-1.5 rounded-lg text-sm font-medium whitespace-nowrap transition-[background-color,box-shadow,transform,color] duration-[var(--motion-default)] ease-[cubic-bezier(0.2,0,0,1)] outline-none select-none focus-visible:outline focus-visible:outline-[var(--focus-ring-width)] focus-visible:outline-offset-[var(--focus-ring-offset)] focus-visible:outline-[var(--focus-ring-color)] disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 active:translate-y-px active:scale-[0.985] hover:shadow-[var(--shadow-xs)]",
  {
    variants: {
      variant: {
        default:
          "bg-transparent text-foreground/70 hover:bg-muted hover:text-foreground data-pressed:bg-accent-soft data-pressed:text-accent data-pressed:border data-pressed:border-accent/20",
        outline:
          "border border-border bg-background text-foreground/70 hover:bg-muted hover:text-foreground data-pressed:bg-accent-soft data-pressed:text-accent data-pressed:border-accent/30",
        solid:
          "bg-muted text-foreground/70 hover:bg-muted/80 hover:text-foreground data-pressed:bg-primary data-pressed:text-primary-foreground data-pressed:shadow-[var(--shadow-sm)]",
      },
      size: {
        default: "h-8 px-2.5",
        sm: "h-7 gap-1 px-2 text-xs rounded-md [&_svg:not([class*='size-'])]:size-3.5",
        lg: "h-9 px-3",
        icon: "size-8",
        "icon-sm": "size-7 rounded-md",
        "icon-lg": "size-9",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

function Toggle({
  className,
  variant,
  size,
  ...props
}: React.ComponentProps<typeof TogglePrimitive.Root> & VariantProps<typeof toggleVariants>) {
  return (
    <TogglePrimitive.Root
      data-slot="toggle"
      data-variant={variant}
      data-size={size}
      className={cn(toggleVariants({ variant, size, className }))}
      {...props}
    />
  );
}

export { Toggle, toggleVariants };
