import type { Channel } from '../../api/types';

export function splitGroupChannels(channels: readonly Channel[]): {
  mainChannel: Channel | null;
  topicChannels: Channel[];
} {
  const sorted = [...channels].sort(
    (a, b) => a.position - b.position || a.name.localeCompare(b.name),
  );
  const mainChannel = sorted[0] ?? null;
  const topicChannels = sorted.slice(1);
  return { mainChannel, topicChannels };
}
