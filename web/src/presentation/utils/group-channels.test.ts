import { describe, expect, it } from 'vitest';
import { splitGroupChannels } from './group-channels';
import type { Channel } from '../../api/types';

const ch = (id: string, name: string, position: number): Channel => ({
  id,
  server_id: 's1',
  type: 'text',
  name,
  position,
});

describe('splitGroupChannels', () => {
  it('treats lowest-position channel as main', () => {
    const { mainChannel, topicChannels } = splitGroupChannels([
      ch('t1', 'pets', 2),
      ch('m1', 'general', 0),
      ch('t2', 'garden', 1),
    ]);
    expect(mainChannel?.id).toBe('m1');
    expect(topicChannels.map((c) => c.id)).toEqual(['t2', 't1']);
  });

  it('returns empty topics when only main exists', () => {
    const { mainChannel, topicChannels } = splitGroupChannels([ch('m1', 'general', 0)]);
    expect(mainChannel?.name).toBe('general');
    expect(topicChannels).toEqual([]);
  });
});
