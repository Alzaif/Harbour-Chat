import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from './App';

vi.mock('./hooks/useChatWebSocket', () => ({
  useChatWebSocket: () => {},
}));

vi.mock('./api/client', () => ({
  api: {
    me: vi.fn().mockResolvedValue({ id: 'dev-user', email: 'dev@test', displayName: 'Dev User' }),
    listServers: vi.fn().mockResolvedValue([]),
    getServer: vi.fn().mockResolvedValue({
      server: { id: '1', name: 'Harbour Home', icon_url: null, owner_user_id: 'system' },
      channels: [{ id: 'c1', server_id: '1', type: 'text', name: 'general', position: 0 }],
      unreadByChannelId: {},
    }),
    listMembers: vi.fn().mockResolvedValue([]),
    listMessages: vi.fn().mockResolvedValue([]),
    markRead: vi.fn().mockResolvedValue({ ok: true }),
    sendMessage: vi.fn(),
    createServer: vi.fn(),
    createChannel: vi.fn(),
    toggleReaction: vi.fn(),
    uploadAttachment: vi.fn(),
    openDm: vi.fn(),
    listPresence: vi.fn().mockResolvedValue([]),
    setPresence: vi.fn().mockResolvedValue({}),
    listTyping: vi.fn().mockResolvedValue([]),
    setTyping: vi.fn().mockResolvedValue([]),
    listVoiceParticipants: vi.fn().mockResolvedValue([]),
    joinVoice: vi.fn(),
    leaveVoice: vi.fn().mockResolvedValue({ ok: true }),
    updateVoiceState: vi.fn(),
  },
}));

describe('App', () => {
  it('renders chat chrome', () => {
    const { unmount } = render(<App />);
    expect(screen.getByText('Chat')).toBeInTheDocument();
    expect(screen.getByText('All apps')).toBeInTheDocument();
    unmount();
  });
});
