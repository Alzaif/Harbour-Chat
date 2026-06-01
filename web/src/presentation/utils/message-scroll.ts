export interface ScrollMetrics {
  scrollTop: number;
  scrollHeight: number;
  clientHeight: number;
}

export function isNearBottom(metrics: ScrollMetrics, threshold = 64): boolean {
  const distance = metrics.scrollHeight - metrics.scrollTop - metrics.clientHeight;
  return distance <= threshold;
}

/** Preserve viewport position after older messages are prepended above the fold. */
export function scrollTopAfterPrepend(
  before: Pick<ScrollMetrics, 'scrollTop' | 'scrollHeight'>,
  after: Pick<ScrollMetrics, 'scrollHeight'>,
): number {
  const delta = after.scrollHeight - before.scrollHeight;
  return before.scrollTop + delta;
}

export function shouldAutoScrollToBottom(
  force: boolean,
  stick: boolean,
): boolean {
  return force || stick;
}
