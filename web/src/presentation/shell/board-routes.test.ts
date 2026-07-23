import { describe, expect, it } from 'vitest';
import { pathForTab, tabAtIndex, tabFromPathname, tabIndex } from './board-routes';

describe('board-routes', () => {
  it('maps pathnames to tabs', () => {
    expect(tabFromPathname('/board/direct')).toBe('direct');
    expect(tabFromPathname('/board/servers/foo')).toBe('servers');
    expect(tabFromPathname('/board/groups/foo')).toBe('servers');
    expect(tabFromPathname('/board/feed')).toBe('feed');
  });

  it('maps tabs to paths', () => {
    expect(pathForTab('direct')).toBe('/direct');
    expect(pathForTab('servers')).toBe('/servers');
    expect(pathForTab('feed')).toBe('/feed');
  });

  it('cycles tab indices for swipe', () => {
    expect(tabAtIndex(tabIndex('feed') + 1)).toBe('direct');
    expect(tabAtIndex(tabIndex('direct') + 1)).toBe('servers');
  });
});
