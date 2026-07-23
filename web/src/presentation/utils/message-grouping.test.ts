import { describe, expect, it } from 'vitest';
import type { Message } from '../../api/types';
import { startsNewGroup } from './message-grouping';

const make = (overrides: Partial<Message>): Message => ({
  id: 'm',
  channel_id: 'c1',
  author_user_id: 'user-a',
  author_display_name: 'Alice',
  content: 'hi',
  created_at: '2026-07-01T12:00:00.000Z',
  edited_at: null,
  deleted_at: null,
  reactions: [],
  ...overrides,
});

describe('startsNewGroup', () => {
  it('starts a group for the first message', () => {
    expect(startsNewGroup(undefined, make({}))).toBe(true);
  });

  it('merges consecutive messages from the same author within 10 minutes', () => {
    const prev = make({ created_at: '2026-07-01T12:00:00.000Z' });
    const curr = make({ created_at: '2026-07-01T12:09:59.000Z' });
    expect(startsNewGroup(prev, curr)).toBe(false);
  });

  it('starts a new group after more than 10 minutes', () => {
    const prev = make({ created_at: '2026-07-01T12:00:00.000Z' });
    const curr = make({ created_at: '2026-07-01T12:10:01.000Z' });
    expect(startsNewGroup(prev, curr)).toBe(true);
  });

  it('starts a new group when the author changes', () => {
    const prev = make({ author_user_id: 'user-a' });
    const curr = make({ author_user_id: 'user-b', created_at: '2026-07-01T12:00:30.000Z' });
    expect(startsNewGroup(prev, curr)).toBe(true);
  });
});
