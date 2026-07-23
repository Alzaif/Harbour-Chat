import { describe, expect, it } from 'vitest';
import { formatKeyCode, isAllowedPushToTalkKey } from './push-to-talk-key';

describe('push-to-talk-key', () => {
  it('formats common key codes for display', () => {
    expect(formatKeyCode('Space')).toBe('Space');
    expect(formatKeyCode('KeyV')).toBe('V');
    expect(formatKeyCode('Digit5')).toBe('5');
    expect(formatKeyCode('ArrowUp')).toBe('Up Arrow');
    expect(formatKeyCode('ShiftLeft')).toBe('Left Shift');
    expect(formatKeyCode('BracketLeft')).toBe('[');
    expect(formatKeyCode('F4')).toBe('F4');
  });

  it('allows space, letter, digit, modifier, and other keys', () => {
    expect(isAllowedPushToTalkKey('Space')).toBe(true);
    expect(isAllowedPushToTalkKey('KeyV')).toBe(true);
    expect(isAllowedPushToTalkKey('Digit5')).toBe(true);
    expect(isAllowedPushToTalkKey('ShiftLeft')).toBe(true);
    expect(isAllowedPushToTalkKey('ArrowUp')).toBe(true);
    expect(isAllowedPushToTalkKey('F4')).toBe(true);
  });

  it('reserves Escape for cancelling capture and rejects empty codes', () => {
    expect(isAllowedPushToTalkKey('Escape')).toBe(false);
    expect(isAllowedPushToTalkKey('')).toBe(false);
  });
});
