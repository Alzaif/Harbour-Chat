import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { UserSettingsProvider } from '../settings/UserSettingsContext';
import { PartySessionProvider } from '../voice/PartySessionContext';
import { PartyBanner } from './PartyBanner';

vi.mock('../../api/client', () => ({
  api: {
    me: vi.fn().mockResolvedValue({ id: 'dev-user', email: 'dev@test', displayName: 'Dev User' }),
    getSettings: vi.fn().mockResolvedValue({ pushToTalk: true, pushToTalkKey: 'KeyV' }),
    updateSettings: vi.fn(),
    listVoiceParticipants: vi.fn().mockResolvedValue([]),
    joinVoice: vi.fn(),
    leaveVoice: vi.fn().mockResolvedValue({ ok: true }),
    updateVoiceState: vi.fn(),
    bootstrapVoiceSession: vi.fn(),
    createVoiceTransport: vi.fn(),
    connectVoiceTransport: vi.fn(),
    createVoiceProducer: vi.fn(),
    listRemoteVoiceProducers: vi.fn().mockResolvedValue({ producers: [] }),
    createVoiceConsumer: vi.fn(),
    closeVoiceSession: vi.fn(),
  },
}));

vi.mock('../../hooks/useChatWebSocket', () => ({
  useChatWebSocket: () => {},
}));

vi.mock('../voice/useVoiceMediaClient', () => ({
  useVoiceMediaClient: () => ({
    connectionState: 'idle',
    localStream: null,
    remoteStreams: [],
    error: null,
    start: vi.fn(),
    stop: vi.fn(),
  }),
}));

function renderBanner() {
  return render(
    <UserSettingsProvider>
      <PartySessionProvider>
        <PartyBanner />
      </PartySessionProvider>
    </UserSettingsProvider>,
  );
}

describe('PartyBanner', () => {
  it('renders nothing when not in a party', () => {
    renderBanner();
    expect(screen.queryByRole('region', { name: 'Active party' })).not.toBeInTheDocument();
  });
});

describe('PartyBanner controls', () => {
  it('shows origin and disconnect when party session is active', async () => {
    const { usePartySession } = await import('../voice/PartySessionContext');

    function Harness() {
      const party = usePartySession();
      return (
        <>
          <button
            type="button"
            onClick={() =>
              void party.joinParty({
                serverId: 'srv-1',
                serverName: 'Test Rook',
                channelId: 'party-1',
                channelName: 'general',
              })
            }
          >
            Join test party
          </button>
          <PartyBanner />
        </>
      );
    }

    render(
      <UserSettingsProvider>
        <PartySessionProvider>
          <Harness />
        </PartySessionProvider>
      </UserSettingsProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Join test party' }));

    expect(await screen.findByText('Test Rook · general')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Disconnect' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Mute' })).toBeInTheDocument();
  });
});
