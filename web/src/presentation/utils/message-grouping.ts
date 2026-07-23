import type { Message } from '../../api/types';

/** Messages from the same author within this window are visually grouped. */
export const GROUP_WINDOW_MS = 10 * 60 * 1000;

/**
 * Whether `curr` should start a new visual group (showing avatar + author + time)
 * rather than being merged under the previous message.
 */
export function startsNewGroup(prev: Message | undefined, curr: Message): boolean {
  if (!prev) return true;
  if (prev.author_user_id !== curr.author_user_id) return true;
  const prevTime = new Date(prev.created_at).getTime();
  const currTime = new Date(curr.created_at).getTime();
  if (Number.isNaN(prevTime) || Number.isNaN(currTime)) return true;
  return currTime - prevTime > GROUP_WINDOW_MS;
}
