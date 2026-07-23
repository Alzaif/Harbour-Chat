const STORAGE_KEY = 'harbour-board-server-last-channel';

type LastChannelMap = Record<string, string>;

function readMap(): LastChannelMap {
  if (typeof window === 'undefined') return {};
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as unknown;
    if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) return {};
    return parsed as LastChannelMap;
  } catch {
    return {};
  }
}

function writeMap(map: LastChannelMap): void {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(map));
}

export function getLastChannelId(serverId: string): string | null {
  const value = readMap()[serverId];
  return typeof value === 'string' && value.length > 0 ? value : null;
}

export function rememberLastChannelId(serverId: string, channelId: string): void {
  const map = readMap();
  map[serverId] = channelId;
  writeMap(map);
}

export function clearLastChannelId(serverId: string): void {
  const map = readMap();
  delete map[serverId];
  writeMap(map);
}

export function resolveChannelForServer(
  serverId: string,
  channelIds: string[],
  preferredChannelId?: string | null,
): string | null {
  const last = getLastChannelId(serverId);
  if (last && channelIds.includes(last)) return last;
  if (preferredChannelId && channelIds.includes(preferredChannelId)) return preferredChannelId;
  return channelIds[0] ?? null;
}
