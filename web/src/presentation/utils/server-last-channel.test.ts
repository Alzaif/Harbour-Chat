import { describe, expect, it, beforeEach } from 'vitest';
import {
  clearLastChannelId,
  getLastChannelId,
  rememberLastChannelId,
  resolveChannelForServer,
} from './server-last-channel';

describe('server-last-channel', () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it('remembers and reads the last channel per server', () => {
    rememberLastChannelId('srv-a', 'ch-1');
    rememberLastChannelId('srv-b', 'ch-9');

    expect(getLastChannelId('srv-a')).toBe('ch-1');
    expect(getLastChannelId('srv-b')).toBe('ch-9');
  });

  it('resolves last channel when still valid, otherwise preferred, otherwise first', () => {
    rememberLastChannelId('srv-a', 'ch-old');
    expect(resolveChannelForServer('srv-a', ['ch-1', 'ch-2'], 'ch-2')).toBe('ch-2');

    rememberLastChannelId('srv-a', 'ch-old');
    expect(resolveChannelForServer('srv-a', ['ch-old', 'ch-2'], 'ch-2')).toBe('ch-old');

    clearLastChannelId('srv-a');
    expect(resolveChannelForServer('srv-a', ['ch-1', 'ch-2'], null)).toBe('ch-1');
  });

  it('clears stored channel for a server', () => {
    rememberLastChannelId('srv-a', 'ch-1');
    clearLastChannelId('srv-a');
    expect(getLastChannelId('srv-a')).toBeNull();
  });
});
