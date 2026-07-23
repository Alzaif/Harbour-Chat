import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { UserSettingsProvider } from '../settings/UserSettingsContext';
import { PartySessionProvider, usePartySession } from './PartySessionContext';

const fakeTrack = { enabled: true, stop: vi.fn() };
const fakeLocalStream = {
  getAudioTracks: () => [fakeTrack],
  getTracks: () => [fakeTrack],
} as unknown as MediaStream;
const fakeRemoteStream = {
  getAudioTracks: () => [],
  getTracks: () => [],
} as unknown as MediaStream;

vi.mock('../../api/client', () => ({
  api: {
    me: vi.fn().mockResolvedValue({ id: 'dev-user', email: 'dev@test', displayName: 'Dev User' }),
    getSettings: vi.fn().mockResolvedValue({ pushToTalk: false, pushToTalkKey: 'Space' }),
    updateSettings: vi.fn(),
    listVoiceParticipants: vi.fn().mockResolvedValue([]),
    joinVoice: vi.fn().mockResolvedValue({
      channel_id: 'party-1',
      user_id: 'dev-user',
      muted: false,
      deafened: false,
      updated_at: '2026-01-01T00:00:00Z',
    }),
    leaveVoice: vi.fn().mockResolvedValue({ ok: true }),
    updateVoiceState: vi.fn().mockResolvedValue({}),
  },
}));

vi.mock('../../hooks/useChatWebSocket', () => ({ useChatWebSocket: () => {} }));

vi.mock('./useVoiceMediaClient', () => ({
  useVoiceMediaClient: () => ({
    connectionState: 'connected',
    localStream: fakeLocalStream,
    remoteStreams: [{ producerId: 'p1', userId: 'other', stream: fakeRemoteStream }],
    error: null,
    start: vi.fn().mockResolvedValue(undefined),
    stop: vi.fn().mockResolvedValue(undefined),
  }),
}));

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
        join
      </button>
      <button type="button" onClick={() => void party.toggleMute()}>
        mute
      </button>
      <button type="button" onClick={() => void party.toggleDeafen()}>
        deafen
      </button>
      <span data-testid="in-party">{String(party.inParty)}</span>
      <span data-testid="muted">{String(party.voiceMuted)}</span>
      <span data-testid="deafened">{String(party.voiceDeafened)}</span>
    </>
  );
}

function renderHarness() {
  return render(
    <UserSettingsProvider>
      <PartySessionProvider>
        <Harness />
      </PartySessionProvider>
    </UserSettingsProvider>,
  );
}

afterEach(() => {
  cleanup();
  fakeTrack.enabled = true;
});

describe('PartySession voice controls', () => {
  it('mute disables and re-enables the local mic track', async () => {
    renderHarness();
    fireEvent.click(screen.getByText('join'));
    await waitFor(() => expect(screen.getByTestId('in-party')).toHaveTextContent('true'));

    fireEvent.click(screen.getByText('mute'));
    expect(await screen.findByTestId('muted')).toHaveTextContent('true');
    expect(fakeTrack.enabled).toBe(false);

    fireEvent.click(screen.getByText('mute'));
    expect(await screen.findByTestId('muted')).toHaveTextContent('false');
    expect(fakeTrack.enabled).toBe(true);
  });

  it('deafen silences remote audio and stops transmitting', async () => {
    renderHarness();
    fireEvent.click(screen.getByText('join'));
    await waitFor(() => expect(screen.getByTestId('in-party')).toHaveTextContent('true'));

    fireEvent.click(screen.getByText('deafen'));
    expect(await screen.findByTestId('deafened')).toHaveTextContent('true');
    expect(fakeTrack.enabled).toBe(false);

    const audios = Array.from(document.querySelectorAll('audio')) as HTMLAudioElement[];
    expect(audios.length).toBeGreaterThan(1);
    expect(audios.every((el) => el.muted)).toBe(true);
  });
});
