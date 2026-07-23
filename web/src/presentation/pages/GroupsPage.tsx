import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { api } from '../../api/client';
import type {
  Channel,
  CurrentUser,
  Message,
  PresenceStatus,
  TypingIndicator,
} from '../../api/types';
import { useChatWebSocket } from '../../hooks/useChatWebSocket';
import { AutoGrowTextarea } from '../components/AutoGrowTextarea';
import { ForwardMessageModal } from '../components/ForwardMessageModal';
import { MessageBubble } from '../components/MessageBubble';
import { PartyMenu } from '../components/PartyMenu';
import { PlusIcon, SendArrowIcon } from '../components/icons';
import { useServerMembers } from '../shell/ServerMembersContext';
import { usePartySession } from '../voice/PartySessionContext';
import { startsNewGroup } from '../utils/message-grouping';
import {
  isNearBottom,
  scrollTopAfterPrepend,
  shouldAutoScrollToBottom,
} from '../utils/message-scroll';
import { rememberLastChannelId } from '../utils/server-last-channel';


export function GroupsPage() {
  const navigate = useNavigate();
  const { serverId: routeServerId, channelId: routeChannelId } = useParams<{
    serverId?: string;
    channelId?: string;
  }>();
  const party = usePartySession();
  const members = useServerMembers();
  const [currentUser, setCurrentUser] = useState<CurrentUser | null>(null);
  const [serverName, setServerName] = useState('Server');
  const [selectedServerId, setSelectedServerId] = useState<string | null>(routeServerId ?? null);
  const [channels, setChannels] = useState<Channel[]>([]);
  const [voiceChannels, setVoiceChannels] = useState<Channel[]>([]);
  const [voiceCountsByChannel, setVoiceCountsByChannel] = useState<Record<string, number>>({});
  const [unreadByChannel, setUnreadByChannel] = useState<Record<string, number>>({});
  const [selectedChannelId, setSelectedChannelId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [presenceByUser, setPresenceByUser] = useState<Record<string, PresenceStatus>>({});
  const [typingIndicators, setTypingIndicators] = useState<TypingIndicator[]>([]);
  const [draft, setDraft] = useState('');
  const [replyingTo, setReplyingTo] = useState<Message | null>(null);
  const [forwarding, setForwarding] = useState<Message | null>(null);
  const [pendingFile, setPendingFile] = useState<File | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingOlder, setLoadingOlder] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [showCreateParty, setShowCreateParty] = useState(false);
  const [newName, setNewName] = useState('');
  const messagesRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const shouldStickToBottomRef = useRef(true);
  const forceScrollToBottomRef = useRef(false);
  const prependAnchorRef = useRef<{ scrollTop: number; scrollHeight: number } | null>(null);
  const typingStopTimeoutRef = useRef<number | null>(null);

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

  const loadServer = useCallback(
    async (serverId: string) => {
      const detail = await api.getServer(serverId);
      setServerName(detail.server.name);
      const textChannels = detail.channels.filter((c) => c.type === 'text');
      const voice = detail.channels.filter((c) => c.type === 'voice');
      setChannels(textChannels);
      setVoiceChannels(voice);
      setUnreadByChannel(detail.unreadByChannelId ?? {});
      const activeChannelId = routeChannelId ?? selectedChannelId;
      if (activeChannelId && !textChannels.some((c) => c.id === activeChannelId)) {
        setSelectedChannelId(null);
        navigate(`/servers/${serverId}`, { replace: true });
      }
    },
    [routeChannelId, selectedChannelId, navigate],
  );

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
    if (routeServerId) setSelectedServerId(routeServerId);
  }, [routeServerId]);

  useEffect(() => {
    if (routeServerId && routeChannelId) {
      rememberLastChannelId(routeServerId, routeChannelId);
    }
  }, [routeServerId, routeChannelId]);

  useEffect(() => {
    if (routeChannelId) {
      setSelectedChannelId(routeChannelId);
      return;
    }
    setSelectedChannelId(null);
  }, [routeChannelId]);

  const loadVoiceCounts = useCallback(async (voiceList: Channel[]) => {
    const counts: Record<string, number> = {};
    await Promise.all(
      voiceList.map(async (channel) => {
        try {
          const participants = await api.listVoiceParticipants(channel.id);
          counts[channel.id] = participants.length;
        } catch {
          counts[channel.id] = 0;
        }
      }),
    );
    setVoiceCountsByChannel(counts);
  }, []);

  useEffect(() => {
    (async () => {
      try {
        setLoading(true);
        const me = await api.me();
        setCurrentUser(me);
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to load');
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  useEffect(() => {
    if (!selectedServerId) return;
    (async () => {
      try {
        await loadServer(selectedServerId);
        await loadPresence(selectedServerId);
        await api.setPresence(selectedServerId, 'online');
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to load server');
      }
    })();
  }, [selectedServerId, loadServer, loadPresence]);

  useEffect(() => {
    if (voiceChannels.length === 0) {
      setVoiceCountsByChannel({});
      return;
    }
    void loadVoiceCounts(voiceChannels);
  }, [voiceChannels, loadVoiceCounts]);

  useEffect(() => {
    if (!selectedChannelId) {
      setMessages([]);
      setTypingIndicators([]);
      setReplyingTo(null);
      return;
    }
    setMessages([]);
    setReplyingTo(null);
    setHasMore(true);
    prependAnchorRef.current = null;
    forceScrollToBottomRef.current = true;
    (async () => {
      try {
        const list = await loadMessages(selectedChannelId);
        await loadTyping(selectedChannelId);
        await markChannelRead(selectedChannelId, list);
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to load messages');
      }
    })();
  }, [selectedChannelId, loadMessages, loadTyping, markChannelRead]);

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

  useEffect(
    () => () => {
      if (typingStopTimeoutRef.current) window.clearTimeout(typingStopTimeoutRef.current);
    },
    [],
  );

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
    }
  });

  const send = async () => {
    if (!selectedChannelId) return;
    if (!draft.trim() && !pendingFile) return;
    const content = draft.trim() || '📎';
    const file = pendingFile;
    const replyId = replyingTo?.id;
    setDraft('');
    setPendingFile(null);
    setReplyingTo(null);
    try {
      const msg = await api.sendMessage(
        selectedChannelId,
        content,
        replyId ? { replyToMessageId: replyId } : undefined,
      );
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
      if (replyId) {
        const previous = messages.find((m) => m.id === replyId);
        if (previous) setReplyingTo(previous);
      }
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

  const createPartyChannel = async () => {
    if (!selectedServerId || !newName.trim()) return;
    try {
      const ch = await api.createChannel(selectedServerId, newName.trim(), 'voice');
      setShowCreateParty(false);
      setNewName('');
      await loadServer(selectedServerId);
      await joinVoice(ch.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Create party failed');
    }
  };

  const startParty = async () => {
    if (!selectedServerId) return;
    try {
      let targetId = voiceChannels[0]?.id;
      if (!targetId) {
        const created = await api.createChannel(selectedServerId, 'Party', 'voice');
        targetId = created.id;
        await loadServer(selectedServerId);
      }
      await joinVoice(targetId);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Start party failed');
    }
  };

  const joinVoice = async (channelId: string) => {
    if (!selectedServerId) return;
    const channel = voiceChannels.find((c) => c.id === channelId);
    try {
      await party.joinParty({
        serverId: selectedServerId,
        serverName,
        channelId,
        channelName: channel?.name ?? 'Party',
      });
      void loadVoiceCounts(voiceChannels);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Join voice failed');
    }
  };

  const leaveVoice = async () => {
    try {
      await party.leaveParty();
      void loadVoiceCounts(voiceChannels);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Leave voice failed');
    }
  };

  const selectedChannel = channels.find((c) => c.id === selectedChannelId);
  const showThread = Boolean(routeChannelId && selectedChannel);
  const partyChannelOptions = useMemo(
    () =>
      voiceChannels.map((channel) => ({
        id: channel.id,
        name: channel.name,
        participantCount: voiceCountsByChannel[channel.id],
      })),
    [voiceChannels, voiceCountsByChannel],
  );
  const visibleTyping = typingIndicators.filter(
    (t) => t.user_id !== currentUser?.id && new Date(t.expires_at).getTime() > Date.now(),
  );

  return (
    <div className={`groups-page servers-thread${showThread ? ' groups-page--thread' : ''}`}>
      {showThread ? (
        <>
          <header className="groups-thread__header">
            <button
              type="button"
              className="groups-back-btn"
              onClick={() => navigate(`/servers/${selectedServerId}`)}
              aria-label="Back to channels"
            >
              ←
            </button>
            <span className="groups-thread__title">{selectedChannel!.name}</span>
            <div className="groups-thread__actions">
              <button
                type="button"
                className={`groups-members-btn${members.membersOpen ? ' groups-members-btn--active' : ''}`}
                onClick={members.toggleMembers}
                aria-pressed={members.membersOpen}
              >
                Members
              </button>
              <PartyMenu
                voiceChannels={partyChannelOptions}
                inVoiceChannelId={party.channelId}
                onStartParty={() => void startParty()}
                onJoinParty={(id) => void joinVoice(id)}
                onCreateParty={() => setShowCreateParty(true)}
                onLeaveParty={() => void leaveVoice()}
              />
            </div>
          </header>
          <div
            ref={messagesRef}
            className="groups-thread__messages"
            aria-live="polite"
            onScroll={onMessagesScroll}
          >
            {loadingOlder && <p className="chat-loading-older">Loading older messages…</p>}
            {messages.map((m, i) => (
              <MessageBubble
                key={m.id}
                message={m}
                currentUser={currentUser}
                onToggleReaction={toggleReaction}
                grouped={!startsNewGroup(messages[i - 1], m)}
                onReply={setReplyingTo}
                onForward={setForwarding}
              />
            ))}
          </div>
          {visibleTyping.length > 0 && (
            <div className="chat-typing-indicator">
              {visibleTyping.map((t) => t.display_name ?? t.user_id).join(', ')} typing...
            </div>
          )}
          {replyingTo ? (
            <div className="chat-reply-bar">
              <div className="chat-reply-bar__meta">
                <span className="chat-reply-bar__label">
                  Replying to {replyingTo.author_display_name ?? replyingTo.author_user_id}
                </span>
                <span className="chat-reply-bar__preview">
                  {replyingTo.deleted_at ? '(deleted)' : replyingTo.content}
                </span>
              </div>
              <button
                type="button"
                className="chat-reply-bar__cancel"
                aria-label="Cancel reply"
                onClick={() => setReplyingTo(null)}
              >
                ×
              </button>
            </div>
          ) : null}
          <form
            className="chat-composer groups-composer"
            onSubmit={(e) => {
              e.preventDefault();
              void send();
            }}
          >
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
              aria-label="Attach file"
              onClick={() => fileInputRef.current?.click()}
              disabled={!selectedChannelId}
            >
              <PlusIcon />
            </button>
            {pendingFile && <span className="chat-pending-file">{pendingFile.name}</span>}
            <AutoGrowTextarea
              value={draft}
              onChange={updateDraft}
              placeholder={`Message ${selectedChannel!.name}`}
              disabled={!selectedChannelId}
              ariaLabel={`Message ${selectedChannel!.name}`}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  void send();
                }
              }}
            />
            <button
              type="submit"
              className="chat-send-btn"
              aria-label="Send message"
              disabled={!selectedChannelId || (!draft.trim() && !pendingFile)}
            >
              <SendArrowIcon />
            </button>
          </form>
        </>
      ) : (
        <p className="board-page__hint servers-thread__placeholder">
          {loading ? 'Loading…' : 'Select a channel on the left to open the chat.'}
        </p>
      )}

      {error && (
        <div className="chat-error" role="alert">
          {error}
          <button type="button" onClick={() => setError(null)}>
            Dismiss
          </button>
        </div>
      )}

      {showCreateParty && (
        <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
          <form
            className="chat-modal"
            onSubmit={(e) => {
              e.preventDefault();
              void createPartyChannel();
            }}
          >
            <h2>Create new Party</h2>
            <p className="party-menu__hint">Voice parties are in preview.</p>
            <input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Party name"
              autoFocus
            />
            <div className="chat-modal__actions">
              <button type="button" onClick={() => setShowCreateParty(false)}>
                Cancel
              </button>
              <button type="submit">Create &amp; join</button>
            </div>
          </form>
        </div>
      )}

      {forwarding ? (
        <ForwardMessageModal
          message={forwarding}
          onClose={() => setForwarding(null)}
          onForwarded={() => setForwarding(null)}
        />
      ) : null}
    </div>
  );
}
