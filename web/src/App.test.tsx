import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { AppRoutes } from './App';

vi.mock('./hooks/useChatWebSocket', () => ({
  useChatWebSocket: () => {},
}));

vi.mock('./api/client', () => ({
  api: {
    me: vi.fn().mockResolvedValue({ id: 'dev-user', email: 'dev@test', displayName: 'Dev User' }),
    getSettings: vi.fn().mockResolvedValue({ pushToTalk: false, pushToTalkKey: 'Space' }),
    updateSettings: vi.fn(),
    listServers: vi.fn().mockResolvedValue([]),
    createServer: vi.fn(),
    updateServer: vi.fn(),
    deleteServer: vi.fn(),
    getServer: vi.fn().mockResolvedValue({
      server: {
        id: '1',
        name: 'Harbour Home',
        description: null,
        icon_url: null,
        cardColor: null,
        owner_user_id: 'system',
      },
      channels: [{ id: 'c1', server_id: '1', type: 'text', name: 'general', position: 0 }],
      unreadByChannelId: {},
    }),
    listMembers: vi.fn().mockResolvedValue([]),
    searchUsers: vi.fn().mockResolvedValue([]),
    addMember: vi.fn(),
    listDmPeers: vi.fn().mockResolvedValue([]),
    listDms: vi.fn().mockResolvedValue([]),
    listMessages: vi.fn().mockResolvedValue([]),
    markRead: vi.fn().mockResolvedValue({ ok: true }),
    sendMessage: vi.fn(),
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
  it('renders Board chrome on direct route', () => {
    render(
      <MemoryRouter initialEntries={['/direct']}>
        <AppRoutes />
      </MemoryRouter>,
    );
    expect(screen.getByText('Board', { selector: '.harbour-chrome__app' })).toBeInTheDocument();
    expect(screen.getByRole('navigation', { name: 'Board sections' })).toBeInTheDocument();
    expect(screen.getByText('All apps')).toBeInTheDocument();
  });
});
