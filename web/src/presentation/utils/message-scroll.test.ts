import { describe, expect, it } from 'vitest';
import {
  isNearBottom,
  scrollTopAfterPrepend,
  shouldAutoScrollToBottom,
} from './message-scroll';

describe('message-scroll', () => {
  it('detects near bottom within threshold', () => {
    expect(isNearBottom({ scrollTop: 936, scrollHeight: 1000, clientHeight: 64 }, 64)).toBe(true);
    expect(isNearBottom({ scrollTop: 800, scrollHeight: 1000, clientHeight: 64 }, 64)).toBe(false);
  });

  it('adjusts scrollTop after prepending content', () => {
    const before = { scrollTop: 120, scrollHeight: 800 };
    const after = { scrollHeight: 1200 };
    expect(scrollTopAfterPrepend(before, after)).toBe(520);
  });

  it('auto-scrolls when forced or sticking', () => {
    expect(shouldAutoScrollToBottom(true, false)).toBe(true);
    expect(shouldAutoScrollToBottom(false, true)).toBe(true);
    expect(shouldAutoScrollToBottom(false, false)).toBe(false);
  });
});
