import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { api } from './api/client';
import type {
  Channel,
  CurrentUser,
  Member,
  Message,
  PresenceState,
  PresenceStatus,
  Server,
  TypingIndicator,
  VoiceParticipant,
} from './api/types';
import { HarbourAppBar } from './components/HarbourAppBar';
import { useChatWebSocket } from './hooks/useChatWebSocket';
import { MembersPanel } from './presentation/components/MembersPanel';
import { MessageBubble } from './presentation/components/MessageBubble';
import { useVoiceMediaClient } from './presentation/voice/useVoiceMediaClient';
import {
  isNearBottom,
  scrollTopAfterPrepend,
  shouldAutoScrollToBottom,
} from './presentation/utils/message-scroll';

const HARBOUR_HOME_ID = '00000000-0000-4000-8000-000000000001';
const shellUrl = import.meta.env.VITE_HARBOUR_SHELL_URL?.trim() || window.location.origin;

export function App() {
  const [currentUser, setCurrentUser] = useState<CurrentUser | null>(null);
  const [servers, setServers] = useState<Server[]>([]);
  const [selectedServerId, setSelectedServerId] = useState<string | null>(HARBOUR_HOME_ID);
  const [channels, setChannels] = useState<Channel[]>([]);
  const [dmChannels, setDmChannels] = useState<Channel[]>([]);
  const [unreadByChannel, setUnreadByChannel] = useState<Record<string, number>>({});
  const [selectedChannelId, setSelectedChannelId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [presenceByUser, setPresenceByUser] = useState<Record<string, PresenceStatus>>({});
  const [typingIndicators, setTypingIndicators] = useState<TypingIndicator[]>([]);
  const [voiceParticipants, setVoiceParticipants] = useState<VoiceParticipant[]>([]);
  const [inVoiceChannelId, setInVoiceChannelId] = useState<string | null>(null);
  const [voiceMuted, setVoiceMuted] = useState(false);
  const [voiceDeafened, setVoiceDeafened] = useState(false);
  const [membersOpen, setMembersOpen] = useState(false);
  const [draft, setDraft] = useState('');
  const [pendingFile, setPendingFile] = useState<File | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingOlder, setLoadingOlder] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [showCreateServer, setShowCreateServer] = useState(false);
  const [showCreateChannel, setShowCreateChannel] = useState(false);
  const [showOpenDm, setShowOpenDm] = useState(false);
  const [newName, setNewName] = useState('');
  const [newChannelType, setNewChannelType] = useState<'text' | 'voice'>('text');
  const [dmUserId, setDmUserId] = useState('');
  const messagesRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const shouldStickToBottomRef = useRef(true);
  const forceScrollToBottomRef = useRef(false);
  const prependAnchorRef = useRef<{ scrollTop: number; scrollHeight: number } | null>(null);
  const typingStopTimeoutRef = useRef<number | null>(null);
  const localAudioRef = useRef<HTMLAudioElement>(null);
  const voiceMedia = useVoiceMediaClient();

  const scrollMessagesToBottom = useCallback((behavior: ScrollBehavior = 'auto') => {
    const el = messagesRef.current;
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior });
  }, []);

  const mergeMessage = useCallback((msg: Message) => {
    setMessages((prev) => {
      const idx = prev.findIndex((m) => m.id === msg.id);
      if (idx >= 0) {
        const next = [...prev];
        next[idx] = msg;
        return next;
      }
      return [...prev, msg];
    });
  }, []);

  const loadServers = useCallback(async () => {
    const list = await api.listServers();
    setServers(list);
    if (!selectedServerId && list[0]) setSelectedServerId(list[0].id);
  }, [selectedServerId]);

  const loadServer = useCallback(
    async (serverId: string) => {
      const detail = await api.getServer(serverId);
      const textChannels = detail.channels.filter((c) => c.type !== 'dm');
      const dms = detail.channels.filter((c) => c.type === 'dm');
      setChannels(textChannels);
      setDmChannels(dms);
      setUnreadByChannel(detail.unreadByChannelId ?? {});
      const hasCurrent = selectedChannelId
        ? [...textChannels, ...dms].some((c) => c.id === selectedChannelId)
        : false;
      if (!hasCurrent) {
        setSelectedChannelId(textChannels[0]?.id ?? null);
      }
    },
    [selectedChannelId],
  );

  const loadMembers = useCallback(async (serverId: string) => {
    const list = await api.listMembers(serverId);
    setMembers(list);
  }, []);

  const loadPresence = useCallback(async (serverId: string) => {
    const list = await api.listPresence(serverId);
    const mapped: Record<string, PresenceStatus> = {};
    for (const p of list) mapped[p.user_id] = p.status;
    setPresenceByUser(mapped);
  }, []);

  const loadTyping = useCallback(async (channelId: string) => {
    const list = await api.listTyping(channelId);
    setTypingIndicators(list);
  }, []);

  const loadVoiceParticipants = useCallback(async (channelId: string) => {
    const list = await api.listVoiceParticipants(channelId);
    setVoiceParticipants(list);
    if (!currentUser) return;
    const me = list.find((p) => p.user_id === currentUser.id);
    if (me) {
      setInVoiceChannelId(channelId);
      setVoiceMuted(me.muted);
      setVoiceDeafened(me.deafened);
    } else {
      setInVoiceChannelId((prev) => (prev === channelId ? null : prev));
    }
  }, [currentUser]);

  const loadMessages = useCallback(async (channelId: string, before?: string) => {
    const list = await api.listMessages(channelId, before);
    if (before) {
      setMessages((prev) => [...list, ...prev]);
      setHasMore(list.length >= 50);
    } else {
      setMessages(list);
      setHasMore(list.length >= 50);
    }
    return list;
  }, []);

  const markChannelRead = useCallback(
    async (channelId: string, msgs: Message[]) => {
      const last = msgs[msgs.length - 1];
      if (!last) return;
      try {
        await api.markRead(channelId, last.id);
        setUnreadByChannel((prev) => ({ ...prev, [channelId]: 0 }));
        if (selectedServerId) {
          const detail = await api.getServer(selectedServerId);
          setUnreadByChannel(detail.unreadByChannelId ?? {});
        }
      } catch {
        /* non-fatal */
      }
    },
    [selectedServerId],
  );

  useEffect(() => {
    (async () => {
      try {
        setLoading(true);
        const [me] = await Promise.all([api.me(), loadServers()]);
        setCurrentUser(me);
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to load');
      } finally {
        setLoading(false);
      }
    })();
  }, [loadServers]);

  useEffect(() => {
    if (!selectedServerId) return;
    (async () => {
      try {
        await loadServer(selectedServerId);
        await loadMembers(selectedServerId);
        await loadPresence(selectedServerId);
        await api.setPresence(selectedServerId, 'online');
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to load server');
      }
    })();
  }, [selectedServerId, loadServer, loadMembers, loadPresence]);

  useEffect(() => {
    if (!selectedChannelId) {
      setMessages([]);
      setTypingIndicators([]);
      setVoiceParticipants([]);
      return;
    }
    setMessages([]);
    setHasMore(true);
    prependAnchorRef.current = null;
    forceScrollToBottomRef.current = true;
    (async () => {
      try {
        const list = await loadMessages(selectedChannelId);
        await loadTyping(selectedChannelId);
        const selected = [...channels, ...dmChannels].find((c) => c.id === selectedChannelId);
        if (selected?.type === 'voice') {
          await loadVoiceParticipants(selectedChannelId);
        } else {
          setVoiceParticipants([]);
        }
        await markChannelRead(selectedChannelId, list);
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to load messages');
      }
    })();
  }, [selectedChannelId, loadMessages, loadTyping, loadVoiceParticipants, markChannelRead, channels, dmChannels]);

  useEffect(() => {
    if (!selectedServerId) return;
    if (import.meta.env.MODE === 'test') return;
    const id = window.setInterval(() => {
      void api.setPresence(selectedServerId, 'online').catch(() => {});
    }, 30000);
    return () => window.clearInterval(id);
  }, [selectedServerId]);

  useEffect(() => {
    const el = messagesRef.current;
    if (!el) return;

    const anchor = prependAnchorRef.current;
    if (anchor) {
      requestAnimationFrame(() => {
        const pane = messagesRef.current;
        if (!pane) return;
        pane.scrollTop = scrollTopAfterPrepend(anchor, pane);
        prependAnchorRef.current = null;
      });
      return;
    }

    if (
      shouldAutoScrollToBottom(
        forceScrollToBottomRef.current,
        shouldStickToBottomRef.current,
      )
    ) {
      requestAnimationFrame(() => scrollMessagesToBottom());
      forceScrollToBottomRef.current = false;
      shouldStickToBottomRef.current = true;
    }
  }, [messages, selectedChannelId, scrollMessagesToBottom]);

  const subscribedIds = useMemo(
    () => (selectedChannelId ? [selectedChannelId] : []),
    [selectedChannelId],
  );

  const refreshMessages = useCallback(async () => {
    if (!selectedChannelId) return;
    const el = messagesRef.current;
    const stick = el ? isNearBottom(el) : false;
    await loadMessages(selectedChannelId);
    if (stick) {
      requestAnimationFrame(() => scrollMessagesToBottom());
    }
  }, [selectedChannelId, loadMessages, scrollMessagesToBottom]);

  useEffect(() => {
    if (!selectedChannelId) return;
    if (import.meta.env.MODE === 'test') return;
    if (typeof window === 'undefined' || typeof document === 'undefined') return;
    let disposed = false;
    const id = window.setInterval(() => {
      if (!disposed && document.visibilityState === 'visible') {
        void refreshMessages();
      }
    }, 2000);
    return () => {
      disposed = true;
      window.clearInterval(id);
    };
  }, [selectedChannelId, refreshMessages]);

  useEffect(() => {
    if (!selectedChannelId) return;
    const selected = [...channels, ...dmChannels].find((c) => c.id === selectedChannelId);
    if (selected?.type !== 'voice') return;
    if (import.meta.env.MODE === 'test') return;
    if (typeof window === 'undefined' || typeof document === 'undefined') return;
    let disposed = false;
    const id = window.setInterval(() => {
      if (!disposed && document.visibilityState === 'visible') {
        void loadVoiceParticipants(selectedChannelId);
      }
    }, 2000);
    return () => {
      disposed = true;
      window.clearInterval(id);
    };
  }, [selectedChannelId, channels, dmChannels, loadVoiceParticipants]);

  useEffect(
    () => () => {
      if (typingStopTimeoutRef.current) window.clearTimeout(typingStopTimeoutRef.current);
    },
    [],
  );

  useEffect(() => {
    const el = localAudioRef.current;
    if (!el) return;
    el.srcObject = voiceMedia.localStream;
  }, [voiceMedia.localStream]);

  useChatWebSocket(subscribedIds, (ev) => {
    if (ev.type === 'message_created' && ev.message.channel_id === selectedChannelId) {
      mergeMessage(ev.message);
    } else if (ev.type === 'message_updated' && ev.message.channel_id === selectedChannelId) {
      mergeMessage(ev.message);
    } else if (
      ev.type === 'message_deleted' &&
      ev.channel_id === selectedChannelId
    ) {
      setMessages((prev) =>
        prev.map((m) =>
          m.id === ev.message_id ? { ...m, deleted_at: new Date().toISOString(), content: '' } : m,
        ),
      );
    } else if (ev.type === 'reaction_updated' && ev.channel_id === selectedChannelId) {
      void refreshMessages();
    } else if (ev.type === 'typing_started' && ev.channel_id === selectedChannelId) {
      setTypingIndicators((prev) => {
        const filtered = prev.filter((t) => t.user_id !== ev.user_id);
        return [
          ...filtered,
          {
            channel_id: ev.channel_id,
            user_id: ev.user_id,
            display_name: ev.display_name,
            expires_at: ev.expires_at,
          },
        ];
      });
    } else if (ev.type === 'typing_stopped' && ev.channel_id === selectedChannelId) {
      setTypingIndicators((prev) => prev.filter((t) => t.user_id !== ev.user_id));
    } else if (ev.type === 'presence_changed') {
      setPresenceByUser((prev) => ({ ...prev, [ev.user_id]: ev.status }));
    } else if (ev.type === 'voice_participant_updated' && ev.channel_id === selectedChannelId) {
      setVoiceParticipants((prev) => {
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
      if (ev.user_id === currentUser?.id) {
        if (ev.connected) {
          setInVoiceChannelId(ev.channel_id);
          setVoiceMuted(ev.muted);
          setVoiceDeafened(ev.deafened);
        } else {
          setInVoiceChannelId((prev) => (prev === ev.channel_id ? null : prev));
        }
      }
    }
  });

  const send = async () => {
    if (!selectedChannelId) return;
    if (!draft.trim() && !pendingFile) return;
    const content = draft.trim() || '📎';
    const file = pendingFile;
    setDraft('');
    setPendingFile(null);
    try {
      const msg = await api.sendMessage(selectedChannelId, content);
      let finalMsg = msg;
      if (file) {
        finalMsg = await api.uploadAttachment(msg.id, file);
      }
      forceScrollToBottomRef.current = true;
      mergeMessage(finalMsg);
      await markChannelRead(selectedChannelId, [...messages, finalMsg]);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Send failed');
      setDraft(content === '📎' ? '' : content);
      if (file) setPendingFile(file);
    }
  };

  const updateDraft = (value: string) => {
    setDraft(value);
    if (!selectedChannelId || !currentUser) return;
    void api.setTyping(selectedChannelId, value.trim().length > 0).catch(() => {});
    if (typingStopTimeoutRef.current) {
      window.clearTimeout(typingStopTimeoutRef.current);
    }
    typingStopTimeoutRef.current = window.setTimeout(() => {
      if (!selectedChannelId) return;
      void api.setTyping(selectedChannelId, false).catch(() => {});
    }, 2500);
  };

  const toggleReaction = async (messageId: string, emoji: string) => {
    try {
      await api.toggleReaction(messageId, emoji);
      await refreshMessages();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Reaction failed');
    }
  };

  const loadOlder = async () => {
    if (!selectedChannelId || loadingOlder || !hasMore || messages.length === 0) return;
    const el = messagesRef.current;
    if (el) {
      prependAnchorRef.current = { scrollTop: el.scrollTop, scrollHeight: el.scrollHeight };
    }
    setLoadingOlder(true);
    try {
      await loadMessages(selectedChannelId, messages[0]!.id);
    } catch (e) {
      prependAnchorRef.current = null;
      setError(e instanceof Error ? e.message : 'Failed to load older messages');
    } finally {
      setLoadingOlder(false);
    }
  };

  const onMessagesScroll = () => {
    const el = messagesRef.current;
    if (!el) return;
    shouldStickToBottomRef.current = isNearBottom(el);
    if (el.scrollTop <= 80) {
      void loadOlder();
    }
  };

  const createServer = async () => {
    if (!newName.trim()) return;
    try {
      const server = await api.createServer(newName.trim());
      setShowCreateServer(false);
      setNewName('');
      await loadServers();
      setSelectedServerId(server.id);
      setSelectedChannelId(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Create server failed');
    }
  };

  const createChannel = async () => {
    if (!selectedServerId || !newName.trim()) return;
    try {
      const ch = await api.createChannel(selectedServerId, newName.trim(), newChannelType);
      setShowCreateChannel(false);
      setNewName('');
      setNewChannelType('text');
      await loadServer(selectedServerId);
      setSelectedChannelId(ch.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Create channel failed');
    }
  };

  const openDm = async () => {
    if (!dmUserId.trim()) return;
    try {
      const ch = await api.openDm(dmUserId.trim());
      setShowOpenDm(false);
      setDmUserId('');
      setDmChannels((prev) => (prev.some((c) => c.id === ch.id) ? prev : [...prev, ch]));
      setSelectedChannelId(ch.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Open DM failed');
    }
  };

  const joinVoice = async (channelId: string) => {
    try {
      await voiceMedia.start(channelId);
      const participant = await api.joinVoice(channelId, voiceMuted, voiceDeafened);
      setInVoiceChannelId(channelId);
      setVoiceParticipants((prev) => [
        ...prev.filter((p) => p.user_id !== participant.user_id),
        participant,
      ]);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Join voice failed');
    }
  };

  const leaveVoice = async () => {
    if (!inVoiceChannelId) return;
    try {
      await api.leaveVoice(inVoiceChannelId);
      await voiceMedia.stop();
      setVoiceParticipants((prev) => prev.filter((p) => p.user_id !== currentUser?.id));
      setInVoiceChannelId(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Leave voice failed');
    }
  };

  const updateVoiceFlags = async (muted: boolean, deafened: boolean) => {
    setVoiceMuted(muted);
    setVoiceDeafened(deafened);
    if (!inVoiceChannelId) return;
    try {
      await api.updateVoiceState(inVoiceChannelId, muted, deafened);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Voice state update failed');
    }
  };

  const selectedChannel = [...channels, ...dmChannels].find((c) => c.id === selectedChannelId);
  const serverName = servers.find((s) => s.id === selectedServerId)?.name ?? 'Harbour Home';
  const visibleTyping = typingIndicators.filter(
    (t) => t.user_id !== currentUser?.id && new Date(t.expires_at).getTime() > Date.now(),
  );

  return (
    <div className="chat-app">
      <HarbourAppBar homeUrl={shellUrl} appName="Chat" />
      {error && (
        <div className="chat-error" role="alert">
          {error}
          <button type="button" onClick={() => setError(null)}>
            Dismiss
          </button>
        </div>
      )}
      <div className={`chat-layout${membersOpen ? ' chat-layout--members' : ''}`}>
        <nav className="chat-servers" aria-label="Servers">
          {servers.map((s) => (
            <button
              key={s.id}
              type="button"
              className={`chat-server-btn${s.id === selectedServerId ? ' is-active' : ''}`}
              title={s.name}
              onClick={() => {
                setSelectedServerId(s.id);
                setSelectedChannelId(null);
                setMembersOpen(false);
              }}
            >
              {s.name.slice(0, 2).toUpperCase()}
            </button>
          ))}
          <button
            type="button"
            className="chat-server-btn chat-server-btn--add"
            title="Create server"
            onClick={() => setShowCreateServer(true)}
          >
            +
          </button>
        </nav>
        <aside className="chat-channels">
          <div className="chat-channels__head">
            <h2 className="chat-channels__title">{serverName}</h2>
            <button
              type="button"
              className="chat-icon-btn"
              title="Create channel"
              onClick={() => {
                setNewChannelType('text');
                setShowCreateChannel(true);
              }}
            >
              +
            </button>
          </div>
          <div className="chat-channels__scroll">
            <ul className="chat-channel-list">
              {channels
                .filter((ch) => ch.type === 'text')
                .map((ch) => (
                <li key={ch.id}>
                  <button
                    type="button"
                    className={`chat-channel-btn${ch.id === selectedChannelId ? ' is-active' : ''}`}
                    onClick={() => {
                      setSelectedChannelId(ch.id);
                      setMembersOpen(false);
                    }}
                  >
                    <span className="chat-channel-hash">#</span> {ch.name}
                    {(unreadByChannel[ch.id] ?? 0) > 0 && (
                      <span className="chat-unread-badge">{unreadByChannel[ch.id]}</span>
                    )}
                  </button>
                </li>
              ))}
            </ul>
            <h3 className="chat-section-label">Voice channels</h3>
            <ul className="chat-channel-list">
              {channels
                .filter((ch) => ch.type === 'voice')
                .map((ch) => (
                  <li key={ch.id}>
                    <button
                      type="button"
                      className={`chat-channel-btn${ch.id === selectedChannelId ? ' is-active' : ''}`}
                      onClick={() => {
                        setSelectedChannelId(ch.id);
                        setMembersOpen(false);
                      }}
                    >
                      <span aria-hidden>🔊</span> {ch.name}
                    </button>
                  </li>
                ))}
            </ul>
            <button
              type="button"
              className="chat-dm-open"
              onClick={() => {
                setNewChannelType('voice');
                setShowCreateChannel(true);
              }}
            >
              Create voice channel
            </button>
            <h3 className="chat-section-label">Direct messages</h3>
            <ul className="chat-channel-list">
              {dmChannels.map((ch) => (
                <li key={ch.id}>
                  <button
                    type="button"
                    className={`chat-channel-btn${ch.id === selectedChannelId ? ' is-active' : ''}`}
                    onClick={() => setSelectedChannelId(ch.id)}
                  >
                    @ {ch.name}
                  </button>
                </li>
              ))}
            </ul>
            <button type="button" className="chat-dm-open" onClick={() => setShowOpenDm(true)}>
              Open DM by user ID
            </button>
          </div>
        </aside>
        <main className="chat-main">
          <header className="chat-main__header">
            <h1>
              {selectedChannel?.type === 'dm' ? '@' : '#'} {selectedChannel?.name ?? 'general'}
            </h1>
            {selectedChannel?.type === 'voice' && selectedChannelId && (
              <div className="chat-voice-controls">
                {inVoiceChannelId === selectedChannelId ? (
                  <>
                    <button type="button" onClick={() => void leaveVoice()}>
                      Leave
                    </button>
                    <button
                      type="button"
                      onClick={() => void updateVoiceFlags(!voiceMuted, voiceDeafened)}
                    >
                      {voiceMuted ? 'Unmute' : 'Mute'}
                    </button>
                    <button
                      type="button"
                      onClick={() => void updateVoiceFlags(voiceMuted, !voiceDeafened)}
                    >
                      {voiceDeafened ? 'Undeafen' : 'Deafen'}
                    </button>
                  </>
                ) : (
                  <button type="button" onClick={() => void joinVoice(selectedChannelId)}>
                    Join Voice
                  </button>
                )}
              </div>
            )}
            {selectedServerId && (
              <button type="button" className="chat-members-toggle" onClick={() => setMembersOpen((o) => !o)}>
                Members
              </button>
            )}
          </header>
          <div
            ref={messagesRef}
            className="chat-messages"
            aria-live="polite"
            onScroll={onMessagesScroll}
          >
            {loadingOlder && <p className="chat-loading-older">Loading older messages…</p>}
            {messages.map((m) => (
              <MessageBubble
                key={m.id}
                message={m}
                currentUser={currentUser}
                onToggleReaction={toggleReaction}
              />
            ))}
          </div>
          {visibleTyping.length > 0 && (
            <div className="chat-typing-indicator">
              {visibleTyping.map((t) => t.display_name ?? t.user_id).join(', ')} typing...
            </div>
          )}
          {selectedChannel?.type === 'voice' && (
            <div className="chat-voice-roster">
              <strong>In voice:</strong>{' '}
              {voiceParticipants.length === 0
                ? 'Nobody yet'
                : voiceParticipants
                    .map((p) => `${p.display_name ?? p.user_id}${p.muted ? ' (muted)' : ''}${p.deafened ? ' (deafened)' : ''}`)
                    .join(', ')}
              <div className="chat-voice-status">
                Media: {voiceMedia.connectionState}
                {voiceMedia.error ? ` - ${voiceMedia.error}` : ''}
              </div>
            </div>
          )}
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
                const playPromise = el.play();
                if (playPromise) {
                  playPromise.catch(() => {
                    // Browser autoplay policy may still block; leave element attached for manual retry.
                  });
                }
              }}
            />
          ))}
          <footer className="chat-composer">
            <input
              ref={fileInputRef}
              type="file"
              accept="image/*,application/pdf"
              hidden
              onChange={(e) => setPendingFile(e.target.files?.[0] ?? null)}
            />
            <button
              type="button"
              className="chat-attach-btn"
              title="Attach file"
              onClick={() => fileInputRef.current?.click()}
              disabled={!selectedChannelId || selectedChannel?.type === 'voice'}
            >
              📎
            </button>
            {pendingFile && <span className="chat-pending-file">{pendingFile.name}</span>}
            <input
              type="text"
              placeholder={`Message #${selectedChannel?.name ?? 'general'}`}
              value={draft}
              onChange={(e) => updateDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  void send();
                }
              }}
              disabled={!selectedChannelId || selectedChannel?.type === 'voice'}
            />
            <button
              type="button"
              onClick={() => void send()}
              disabled={
                !selectedChannelId ||
                selectedChannel?.type === 'voice' ||
                (!draft.trim() && !pendingFile)
              }
            >
              Send
            </button>
          </footer>
        </main>
        <MembersPanel
          members={members}
          presenceByUser={presenceByUser}
          open={membersOpen}
          onClose={() => setMembersOpen(false)}
        />
      </div>

      {showCreateServer && (
        <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
          <form
            className="chat-modal"
            onSubmit={(e) => {
              e.preventDefault();
              void createServer();
            }}
          >
            <h2>Create server</h2>
            <input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Server name"
              autoFocus
            />
            <div className="chat-modal__actions">
              <button type="button" onClick={() => setShowCreateServer(false)}>
                Cancel
              </button>
              <button type="submit">Create</button>
            </div>
          </form>
        </div>
      )}

      {showCreateChannel && (
        <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
          <form
            className="chat-modal"
            onSubmit={(e) => {
              e.preventDefault();
              void createChannel();
            }}
          >
            <h2>Create channel</h2>
            <input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Channel name"
              autoFocus
            />
            <label className="chat-modal__field">
              <span>Type</span>
              <select
                value={newChannelType}
                onChange={(e) => setNewChannelType(e.target.value as 'text' | 'voice')}
              >
                <option value="text">Text</option>
                <option value="voice">Voice</option>
              </select>
            </label>
            <div className="chat-modal__actions">
              <button
                type="button"
                onClick={() => {
                  setShowCreateChannel(false);
                  setNewChannelType('text');
                }}
              >
                Cancel
              </button>
              <button type="submit">Create</button>
            </div>
          </form>
        </div>
      )}

      {showOpenDm && (
        <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
          <form
            className="chat-modal"
            onSubmit={(e) => {
              e.preventDefault();
              void openDm();
            }}
          >
            <h2>Open direct message</h2>
            <input
              value={dmUserId}
              onChange={(e) => setDmUserId(e.target.value)}
              placeholder="Harbour user ID"
              autoFocus
            />
            <div className="chat-modal__actions">
              <button type="button" onClick={() => setShowOpenDm(false)}>
                Cancel
              </button>
              <button type="submit">Open</button>
            </div>
          </form>
        </div>
      )}

      {loading && <p className="chat-loading">Loading…</p>}
    </div>
  );
}
