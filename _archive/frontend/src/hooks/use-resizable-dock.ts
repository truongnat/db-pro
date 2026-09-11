import { useCallback, useEffect, useRef, useState } from "react";

interface UseResizableDockOptions {
  initialRatio?: number;
  minTop?: number;
  minBottom?: number;
  storageKey?: string;
}

interface SeparatorProps {
  onMouseDown: (e: React.MouseEvent) => void;
  onDoubleClick: () => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  role: "separator";
  "aria-orientation": "horizontal";
  "aria-valuenow": number;
  "aria-valuemin": number;
  "aria-valuemax": number;
  tabIndex: 0;
}

interface UseResizableDockReturn {
  topHeight: number;
  separatorProps: SeparatorProps;
  isCollapsed: boolean;
  toggleCollapse: () => void;
  containerRef: React.RefObject<HTMLDivElement | null>;
}

const DEFAULT_STORAGE_KEY = "db-pro-dock-size";
const DEFAULT_INITIAL_RATIO = 0.65;
const DEFAULT_MIN_TOP = 100;
const DEFAULT_MIN_BOTTOM = 150;
const KEYBOARD_STEP = 0.01;

function loadStoredRatio(storageKey: string): number | null {
  try {
    const stored = localStorage.getItem(storageKey);
    if (stored === null) return null;
    const ratio = parseFloat(stored);
    return Number.isFinite(ratio) && ratio > 0 && ratio < 1 ? ratio : null;
  } catch {
    return null;
  }
}

function saveRatio(storageKey: string, ratio: number) {
  try {
    localStorage.setItem(storageKey, ratio.toString());
  } catch {
    // ignore storage errors
  }
}

export function useResizableDock(options: UseResizableDockOptions = {}): UseResizableDockReturn {
  const {
    initialRatio = DEFAULT_INITIAL_RATIO,
    minTop = DEFAULT_MIN_TOP,
    minBottom = DEFAULT_MIN_BOTTOM,
    storageKey = DEFAULT_STORAGE_KEY,
  } = options;

  const containerRef = useRef<HTMLDivElement | null>(null);
  const [ratio, setRatio] = useState<number>(() => {
    return loadStoredRatio(storageKey) ?? initialRatio;
  });
  const [collapsed, setCollapsed] = useState(false);
  const [containerHeight, setContainerHeight] = useState(0);
  const lastRatioRef = useRef(ratio);
  const draggingRef = useRef(false);

  const clampRatio = useCallback(
    (r: number, height: number) => {
      if (height <= 0) return r;
      const minR = minTop / height;
      const maxR = 1 - minBottom / height;
      if (minR >= maxR) return r;
      return Math.max(minR, Math.min(maxR, r));
    },
    [minTop, minBottom],
  );

  const topHeight = collapsed ? 0 : Math.round(ratio * containerHeight);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      draggingRef.current = true;

      const container = containerRef.current;
      if (!container) return;

      const handleMouseMove = (moveEvent: MouseEvent) => {
        if (!draggingRef.current || !container) return;
        const rect = container.getBoundingClientRect();
        const y = moveEvent.clientY - rect.top;
        const newRatio = clampRatio(y / rect.height, rect.height);
        setRatio(newRatio);
        setCollapsed(false);
      };

      const handleMouseUp = () => {
        draggingRef.current = false;
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
        setRatio((currentRatio) => {
          lastRatioRef.current = currentRatio;
          saveRatio(storageKey, currentRatio);
          return currentRatio;
        });
      };

      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
    },
    [clampRatio, storageKey],
  );

  const toggleCollapse = useCallback(() => {
    setCollapsed((prev) => {
      if (prev) {
        setRatio(lastRatioRef.current);
        return false;
      } else {
        setRatio((r) => {
          lastRatioRef.current = r;
          return 0;
        });
        return true;
      }
    });
  }, []);

  const handleDoubleClick = useCallback(() => {
    // Reset to default ratio on double-click
    setRatio(initialRatio);
    setCollapsed(false);
    saveRatio(storageKey, initialRatio);
    lastRatioRef.current = initialRatio;
  }, [initialRatio, storageKey]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      let newRatio = ratio;
      const height = containerHeight;

      switch (e.key) {
        case "ArrowUp":
          e.preventDefault();
          newRatio = clampRatio(ratio - KEYBOARD_STEP, height);
          break;
        case "ArrowDown":
          e.preventDefault();
          newRatio = clampRatio(ratio + KEYBOARD_STEP, height);
          break;
        case "Home":
          e.preventDefault();
          newRatio = clampRatio(0, height);
          break;
        case "End":
          e.preventDefault();
          newRatio = clampRatio(1, height);
          break;
        default:
          return;
      }

      setRatio(newRatio);
      setCollapsed(false);
      lastRatioRef.current = newRatio;
      saveRatio(storageKey, newRatio);
    },
    [ratio, containerHeight, clampRatio, storageKey],
  );

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const updateHeight = () => {
      const h = container.clientHeight;
      setContainerHeight((prev) => (prev !== h ? h : prev));
    };

    updateHeight();

    const observer = new ResizeObserver(updateHeight);
    observer.observe(container);
    return () => observer.disconnect();
  }, []);

  const minRatio = containerHeight > 0 ? Math.round((minTop / containerHeight) * 100) : 0;
  const maxRatio = containerHeight > 0 ? Math.round((1 - minBottom / containerHeight) * 100) : 100;

  return {
    topHeight,
    separatorProps: {
      onMouseDown: handleMouseDown,
      onDoubleClick: handleDoubleClick,
      onKeyDown: handleKeyDown,
      role: "separator",
      "aria-orientation": "horizontal",
      "aria-valuenow": Math.round(ratio * 100),
      "aria-valuemin": minRatio,
      "aria-valuemax": maxRatio,
      tabIndex: 0,
    },
    isCollapsed: collapsed,
    toggleCollapse,
    containerRef,
  };
}
