import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { api } from '../../api/client';
import type { VoiceParticipant } from '../../api/types';
import { useChatWebSocket } from '../../hooks/useChatWebSocket';
import { useUserSettings } from '../settings/UserSettingsContext';
import { usePushToTalk } from './usePushToTalk';
import { useVoiceMediaClient } from './useVoiceMediaClient';

export interface PartyOrigin {
  serverId: string;
  serverName: string;
  channelId: string;
  channelName: string;
}

interface PartySessionContextValue {
  inParty: boolean;
  origin: PartyOrigin | null;
  channelId: string | null;
  voiceMuted: boolean;
  voiceDeafened: boolean;
  connectionState: string;
  error: string | null;
  participants: VoiceParticipant[];
  joinParty: (origin: PartyOrigin) => Promise<void>;
  leaveParty: () => Promise<void>;
  toggleMute: () => Promise<void>;
  toggleDeafen: () => Promise<void>;
}

const PartySessionContext = createContext<PartySessionContextValue | null>(null);

export function PartySessionProvider({ children }: { children: React.ReactNode }) {
  const { settings } = useUserSettings();
  const voiceMedia = useVoiceMediaClient();
  const [origin, setOrigin] = useState<PartyOrigin | null>(null);
  const [channelId, setChannelId] = useState<string | null>(null);
  const [voiceMuted, setVoiceMuted] = useState(false);
  const [voiceDeafened, setVoiceDeafened] = useState(false);
  const [participants, setParticipants] = useState<VoiceParticipant[]>([]);
  const [currentUserId, setCurrentUserId] = useState<string | null>(null);
  const localAudioRef = useRef<HTMLAudioElement>(null);

  useEffect(() => {
    void api.me().then((user) => setCurrentUserId(user.id)).catch(() => setCurrentUserId(null));
  }, []);

  useEffect(() => {
    const el = localAudioRef.current;
    if (!el) return;
    el.srcObject = voiceMedia.localStream;
  }, [voiceMedia.localStream]);

  // Keep the mic track in sync with mute/deafen, including right after the
  // stream becomes available on join.
  useEffect(() => {
    const track = voiceMedia.localStream?.getAudioTracks()[0];
    if (track) track.enabled = !voiceMuted && !voiceDeafened;
  }, [voiceMedia.localStream, voiceMuted, voiceDeafened]);

  const updateVoiceFlags = useCallback(
    async (muted: boolean, deafened: boolean) => {
      setVoiceMuted(muted);
      setVoiceDeafened(deafened);
      // Apply to the live mic track immediately; a deafened user never transmits.
      const track = voiceMedia.localStream?.getAudioTracks()[0];
      if (track) track.enabled = !muted && !deafened;
      if (!channelId) return;
      try {
        await api.updateVoiceState(channelId, muted, deafened);
      } catch {
        /* non-fatal */
      }
    },
    [channelId, voiceMedia],
  );

  const joinParty = useCallback(
    async (nextOrigin: PartyOrigin) => {
      const startMuted = settings.pushToTalk || voiceMuted;
      await voiceMedia.start(nextOrigin.channelId);
      const participant = await api.joinVoice(nextOrigin.channelId, startMuted, voiceDeafened);
      setOrigin(nextOrigin);
      setChannelId(nextOrigin.channelId);
      setVoiceMuted(startMuted);
      setParticipants((prev) => [
        ...prev.filter((p) => p.user_id !== participant.user_id),
        participant,
      ]);
      const list = await api.listVoiceParticipants(nextOrigin.channelId);
      setParticipants(list);
    },
    [settings.pushToTalk, voiceDeafened, voiceMedia, voiceMuted],
  );

  const leaveParty = useCallback(async () => {
    if (!channelId) return;
    try {
      await api.leaveVoice(channelId);
      await voiceMedia.stop();
    } finally {
      setParticipants([]);
      setOrigin(null);
      setChannelId(null);
      setVoiceMuted(false);
      setVoiceDeafened(false);
    }
  }, [channelId, voiceMedia]);

  const toggleMute = useCallback(async () => {
    await updateVoiceFlags(!voiceMuted, voiceDeafened);
  }, [updateVoiceFlags, voiceDeafened, voiceMuted]);

  const toggleDeafen = useCallback(async () => {
    await updateVoiceFlags(voiceMuted, !voiceDeafened);
  }, [updateVoiceFlags, voiceDeafened, voiceMuted]);

  const handlePushToTalk = useCallback(
    (transmitting: boolean) => {
      if (!settings.pushToTalk || !channelId || voiceDeafened) return;
      const track = voiceMedia.localStream?.getAudioTracks()[0];
      if (track) track.enabled = transmitting;
      void updateVoiceFlags(!transmitting, voiceDeafened);
    },
    [settings.pushToTalk, channelId, voiceDeafened, voiceMedia.localStream, updateVoiceFlags],
  );

  usePushToTalk({
    enabled: settings.pushToTalk,
    active: Boolean(channelId),
    keyCode: settings.pushToTalkKey || 'Space',
    onTransmitChange: handlePushToTalk,
  });

  const subscribedIds = useMemo(() => (channelId ? [channelId] : []), [channelId]);

  useChatWebSocket(subscribedIds, (ev) => {
    if (ev.type !== 'voice_participant_updated' || ev.channel_id !== channelId) return;
    setParticipants((prev) => {
      if (!ev.connected) return prev.filter((p) => p.user_id !== ev.user_id);
      const next = prev.filter((p) => p.user_id !== ev.user_id);
      next.push({
        channel_id: ev.channel_id,
        user_id: ev.user_id,
        display_name: ev.display_name,
        muted: ev.muted,
        deafened: ev.deafened,
        updated_at: ev.updated_at,
      });
      return next;
    });
    if (ev.user_id === currentUserId) {
      if (ev.connected) {
        setChannelId(ev.channel_id);
        setVoiceMuted(ev.muted);
        setVoiceDeafened(ev.deafened);
      }
    }
  });

  useEffect(() => {
    if (!channelId || import.meta.env.MODE === 'test') return;
    const id = window.setInterval(() => {
      void api.listVoiceParticipants(channelId).then(setParticipants).catch(() => {});
    }, 5000);
    return () => window.clearInterval(id);
  }, [channelId]);

  const value = useMemo(
    () => ({
      inParty: Boolean(channelId),
      origin,
      channelId,
      voiceMuted,
      voiceDeafened,
      connectionState: voiceMedia.connectionState,
      error: voiceMedia.error,
      participants,
      joinParty,
      leaveParty,
      toggleMute,
      toggleDeafen,
    }),
    [
      channelId,
      origin,
      voiceMuted,
      voiceDeafened,
      voiceMedia.connectionState,
      voiceMedia.error,
      participants,
      joinParty,
      leaveParty,
      toggleMute,
      toggleDeafen,
    ],
  );

  return (
    <PartySessionContext.Provider value={value}>
      {children}
      <audio ref={localAudioRef} autoPlay muted hidden />
      {voiceMedia.remoteStreams.map((remote) => (
        <audio
          key={remote.producerId}
          autoPlay
          controls={false}
          hidden
          ref={(el) => {
            if (!el) return;
            el.srcObject = remote.stream;
            el.muted = voiceDeafened;
            const playPromise = el.play();
            if (playPromise) playPromise.catch(() => {});
          }}
        />
      ))}
    </PartySessionContext.Provider>
  );
}

export function usePartySession() {
  const ctx = useContext(PartySessionContext);
  if (!ctx) throw new Error('usePartySession must be used within PartySessionProvider');
  return ctx;
}
