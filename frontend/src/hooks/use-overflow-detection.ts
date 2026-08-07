import { useCallback, useEffect, useRef, useState } from "react";

interface OverflowState {
  isOverflowing: boolean;
  canScrollLeft: boolean;
  canScrollRight: boolean;
}

export function useOverflowDetection() {
  const containerRef = useRef<HTMLDivElement>(null);
  const [state, setState] = useState<OverflowState>({
    isOverflowing: false,
    canScrollLeft: false,
    canScrollRight: false,
  });

  const update = useCallback(() => {
    const el = containerRef.current;
    if (!el) return;
    const isOverflowing = el.scrollWidth > el.clientWidth;
    setState({
      isOverflowing,
      canScrollLeft: isOverflowing && el.scrollLeft > 1,
      canScrollRight: isOverflowing && el.scrollLeft + el.clientWidth < el.scrollWidth - 1,
    });
  }, []);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    update();

    const observer = new ResizeObserver(update);
    observer.observe(el);
    el.addEventListener("scroll", update, { passive: true });

    return () => {
      observer.disconnect();
      el.removeEventListener("scroll", update);
    };
  }, [update]);

  const scrollLeft = useCallback(() => {
    containerRef.current?.scrollBy({ left: -200, behavior: "smooth" });
  }, []);

  const scrollRight = useCallback(() => {
    containerRef.current?.scrollBy({ left: 200, behavior: "smooth" });
  }, []);

  return { containerRef, ...state, scrollLeft, scrollRight };
}
