import { Button } from "@/components/ui/button";

interface ZoomControlsProps {
  zoom: number;
  onZoomChange: (zoom: number) => void;
}

const MIN_ZOOM = 50;
const MAX_ZOOM = 200;
const STEP = 10;

export function ZoomControls({ zoom, onZoomChange }: ZoomControlsProps) {
  return (
    <div className="flex items-center gap-1">
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="rounded px-1.5 py-0.5 text-xs text-[var(--app-text-muted)]"
        onClick={() => onZoomChange(Math.max(MIN_ZOOM, zoom - STEP))}
        disabled={zoom <= MIN_ZOOM}
        title="Alt+−"
      >
        −
      </Button>
      <span className="min-w-[3ch] text-center text-xs text-[var(--app-text-muted)]">{zoom}%</span>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="rounded px-1.5 py-0.5 text-xs text-[var(--app-text-muted)]"
        onClick={() => onZoomChange(Math.min(MAX_ZOOM, zoom + STEP))}
        disabled={zoom >= MAX_ZOOM}
        title="Alt++"
      >
        +
      </Button>
    </div>
  );
}
