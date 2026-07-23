import { useCallback, useRef, type PointerEvent } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  pathForTab,
  tabAtIndex,
  tabFromPathname,
  tabIndex,
  type BoardTab,
} from './board-routes';

const SWIPE_THRESHOLD_PX = 50;

function swipeEnabled(): boolean {
  if (typeof window === 'undefined') return false;
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return false;
  return !window.matchMedia('(pointer: fine) and (hover: hover)').matches;
}

export function useSwipeNavigation() {
  const navigate = useNavigate();
  const startRef = useRef<{ x: number; y: number } | null>(null);

  const onPointerDown = useCallback((e: PointerEvent) => {
    if (!swipeEnabled() || e.button !== 0) return;
    startRef.current = { x: e.clientX, y: e.clientY };
  }, []);

  const onPointerUp = useCallback(
    (e: PointerEvent) => {
      if (!swipeEnabled() || !startRef.current) return;
      const dx = e.clientX - startRef.current.x;
      const dy = e.clientY - startRef.current.y;
      startRef.current = null;
      if (Math.abs(dx) < SWIPE_THRESHOLD_PX || Math.abs(dx) < Math.abs(dy)) return;

      const tab = tabFromPathname(window.location.pathname);
      const idx = tabIndex(tab);
      const next: BoardTab = dx < 0 ? tabAtIndex(idx + 1) : tabAtIndex(idx - 1);
      navigate(pathForTab(next));
    },
    [navigate],
  );

  const onPointerCancel = useCallback(() => {
    startRef.current = null;
  }, []);

  return { onPointerDown, onPointerUp, onPointerCancel };
}
