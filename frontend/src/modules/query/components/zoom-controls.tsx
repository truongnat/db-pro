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
      <button
        className="rounded px-1.5 py-0.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
        style={{ color: "var(--color-text-secondary)" }}
        onClick={() => onZoomChange(Math.max(MIN_ZOOM, zoom - STEP))}
        disabled={zoom <= MIN_ZOOM}
        title="Alt+−"
      >
        −
      </button>
      <span className="min-w-[3ch] text-center text-xs" style={{ color: "var(--color-text-secondary)" }}>
        {zoom}%
      </span>
      <button
        className="rounded px-1.5 py-0.5 text-xs transition-colors hover:bg-[var(--color-bg)]"
        style={{ color: "var(--color-text-secondary)" }}
        onClick={() => onZoomChange(Math.min(MAX_ZOOM, zoom + STEP))}
        disabled={zoom >= MAX_ZOOM}
        title="Alt++"
      >
        +
      </button>
    </div>
  );
}
