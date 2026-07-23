export const BOARD_TABS = ['direct', 'servers', 'feed'] as const;
export type BoardTab = (typeof BOARD_TABS)[number];

export const TAB_DISPLAY: Record<BoardTab, string> = {
  direct: 'Direct',
  servers: 'Servers',
  feed: 'Board',
};

export function tabFromPathname(pathname: string): BoardTab {
  if (pathname.includes('/direct')) return 'direct';
  if (pathname.includes('/servers') || pathname.includes('/groups')) return 'servers';
  return 'feed';
}

export function pathForTab(tab: BoardTab): string {
  if (tab === 'feed') return '/feed';
  return `/${tab}`;
}

export function tabIndex(tab: BoardTab): number {
  return BOARD_TABS.indexOf(tab);
}

export function tabAtIndex(index: number): BoardTab {
  const i = ((index % BOARD_TABS.length) + BOARD_TABS.length) % BOARD_TABS.length;
  return BOARD_TABS[i]!;
}
