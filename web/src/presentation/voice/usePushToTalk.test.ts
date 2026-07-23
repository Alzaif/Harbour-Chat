import { renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { usePushToTalk } from './usePushToTalk';

describe('usePushToTalk', () => {
  it('calls onTransmitChange when Space is pressed and released', () => {
    const onTransmitChange = vi.fn();
    renderHook(() =>
      usePushToTalk({
        enabled: true,
        active: true,
        keyCode: 'Space',
        onTransmitChange,
      }),
    );

    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Space' }));
    expect(onTransmitChange).toHaveBeenCalledWith(true);

    window.dispatchEvent(new KeyboardEvent('keyup', { code: 'Space' }));
    expect(onTransmitChange).toHaveBeenCalledWith(false);
  });

  it('ignores Space when disabled', () => {
    const onTransmitChange = vi.fn();
    renderHook(() =>
      usePushToTalk({
        enabled: false,
        active: true,
        keyCode: 'Space',
        onTransmitChange,
      }),
    );

    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Space' }));
    expect(onTransmitChange).not.toHaveBeenCalled();
  });
});
