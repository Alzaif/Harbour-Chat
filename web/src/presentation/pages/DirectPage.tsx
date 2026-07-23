import { useCallback, useEffect, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { api } from '../../api/client';
import type { CurrentUser, DmInboxEntry, DmPeer, Message } from '../../api/types';
import { useChatWebSocket } from '../../hooks/useChatWebSocket';
import { AutoGrowTextarea } from '../components/AutoGrowTextarea';
import { ForwardMessageModal } from '../components/ForwardMessageModal';
import { MessageBubble } from '../components/MessageBubble';
import { SendArrowIcon } from '../components/icons';
import { startsNewGroup } from '../utils/message-grouping';

export function DirectPage() {
  const { channelId } = useParams<{ channelId?: string }>();
  const navigate = useNavigate();
  const [currentUser, setCurrentUser] = useState<CurrentUser | null>(null);
  const [inbox, setInbox] = useState<DmInboxEntry[]>([]);
  const [peers, setPeers] = useState<DmPeer[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState('');
  const [replyingTo, setReplyingTo] = useState<Message | null>(null);
  const [forwarding, setForwarding] = useState<Message | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showNewDm, setShowNewDm] = useState(false);
  const [dmSearchQuery, setDmSearchQuery] = useState('');
  const [dmSearchResults, setDmSearchResults] = useState<DmPeer[]>([]);
  const [dmSearching, setDmSearching] = useState(false);
  const messagesRef = useRef<HTMLDivElement>(null);

  const activeEntry = inbox.find((e) => e.channelId === channelId);

  const loadInbox = useCallback(async () => {
    const list = await api.listDms();
    setInbox(Array.isArray(list) ? list : []);
  }, []);

  const loadThread = useCallback(async (id: string) => {
    const list = await api.listMessages(id);
    setMessages(Array.isArray(list) ? list : []);
    if (Array.isArray(list) && list.length > 0) {
      await api.markRead(id, list[list.length - 1]!.id);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const me = await api.me();
        if (cancelled) return;
        setCurrentUser(me);
        await loadInbox();
        const peerList = await api.listDmPeers();
        if (!cancelled) setPeers(peerList);
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : 'Failed to load');
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [loadInbox]);

  useEffect(() => {
    if (!channelId) {
      setMessages([]);
      setReplyingTo(null);
      return;
    }
    setReplyingTo(null);
    void loadThread(channelId).catch((e) =>
      setError(e instanceof Error ? e.message : 'Failed to load messages'),
    );
  }, [channelId, loadThread]);

  useEffect(() => {
    if (!showNewDm) {
      setDmSearchQuery('');
      setDmSearchResults([]);
      return;
    }
    const trimmed = dmSearchQuery.trim();
    if (trimmed.length < 2) {
      setDmSearchResults([]);
      setDmSearching(false);
      return;
    }
    const handle = window.setTimeout(() => {
      setDmSearching(true);
      void api
        .searchUsers(trimmed)
        .then(setDmSearchResults)
        .catch(() => setDmSearchResults([]))
        .finally(() => setDmSearching(false));
    }, 300);
    return () => window.clearTimeout(handle);
  }, [showNewDm, dmSearchQuery]);

  useChatWebSocket(channelId ? [channelId] : [], (ev) => {
    if (ev.type === 'message_created' && ev.message.channel_id === channelId) {
      setMessages((prev) => [...prev, ev.message]);
      void loadInbox();
    }
  });

  useEffect(() => {
    const el = messagesRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages]);

  const openConversation = (id: string) => {
    navigate(`/direct/${id}`);
  };

  const startDm = async (peerId: string) => {
    try {
      const channel = await api.openDm(peerId);
      setShowNewDm(false);
      await loadInbox();
      navigate(`/direct/${channel.id}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Could not open conversation');
    }
  };

  const send = async () => {
    const text = draft.trim();
    if (!text || !channelId) return;
    const replyId = replyingTo?.id;
    setDraft('');
    setReplyingTo(null);
    try {
      const msg = await api.sendMessage(
        channelId,
        text,
        replyId ? { replyToMessageId: replyId } : undefined,
      );
      setMessages((prev) => [...prev, msg]);
      await loadInbox();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Send failed');
    }
  };

  const showThread = Boolean(channelId);

  return (
    <div className={`direct-page${showThread ? ' direct-page--thread' : ''}`}>
      <aside className={`direct-inbox${showThread ? ' direct-inbox--hidden-mobile' : ''}`}>
        <header className="direct-inbox__header">
          <h1>Direct</h1>
          <button type="button" className="direct-new-btn" onClick={() => setShowNewDm(true)}>
            New
          </button>
        </header>
        {loading ? (
          <p className="board-page__hint">Loading…</p>
        ) : inbox.length === 0 ? (
          <p className="board-page__hint">No conversations yet. Start one with New.</p>
        ) : (
          <ul className="direct-inbox__list">
            {inbox.map((entry) => (
              <li key={entry.channelId}>
                <button
                  type="button"
                  className={`direct-inbox__item${entry.channelId === channelId ? ' direct-inbox__item--active' : ''}`}
                  onClick={() => openConversation(entry.channelId)}
                >
                  <span className="direct-inbox__name">
                    {entry.otherDisplayName ?? entry.otherUserId}
                  </span>
                  {entry.lastMessagePreview && (
                    <span className="direct-inbox__preview">{entry.lastMessagePreview}</span>
                  )}
                  {entry.unreadCount > 0 && (
                    <span className="direct-inbox__badge">{entry.unreadCount}</span>
                  )}
                </button>
              </li>
            ))}
          </ul>
        )}
      </aside>

      <section className={`direct-thread${showThread ? '' : ' direct-thread--hidden-mobile'}`}>
        {showThread ? (
          <>
            <header className="direct-thread__header">
              <button
                type="button"
                className="direct-back-btn"
                onClick={() => navigate('/direct')}
                aria-label="Back to inbox"
              >
                ←
              </button>
              <span>
                {activeEntry?.otherDisplayName ?? activeEntry?.otherUserId ?? 'Conversation'}
              </span>
            </header>
            <div className="direct-thread__messages" ref={messagesRef}>
              {messages.map((m, i) => (
                <MessageBubble
                  key={m.id}
                  message={m}
                  currentUser={currentUser}
                  onToggleReaction={(messageId, emoji) => void api.toggleReaction(messageId, emoji)}
                  grouped={!startsNewGroup(messages[i - 1], m)}
                  onReply={setReplyingTo}
                  onForward={setForwarding}
                />
              ))}
            </div>
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
              className="chat-composer direct-composer"
              onSubmit={(e) => {
                e.preventDefault();
                void send();
              }}
            >
              <AutoGrowTextarea
                value={draft}
                onChange={setDraft}
                placeholder="Message…"
                ariaLabel="Message"
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
                disabled={!draft.trim()}
              >
                <SendArrowIcon />
              </button>
            </form>
          </>
        ) : (
          <p className="board-page__hint direct-thread__placeholder">
            Select a conversation or start a new one.
          </p>
        )}
      </section>

      {error && (
        <div className="chat-error" role="alert">
          {error}
          <button type="button" onClick={() => setError(null)}>
            Dismiss
          </button>
        </div>
      )}

      {showNewDm && (
        <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
          <div className="chat-modal">
            <h2>New message</h2>
            <p className="add-member-modal__hint">
              Search Harbour users by name or email, or pick someone you already share a group with.
            </p>
            <input
              value={dmSearchQuery}
              onChange={(e) => setDmSearchQuery(e.target.value)}
              placeholder="Search by name or email…"
              autoFocus
              aria-label="Search users"
            />
            {dmSearching && <p className="board-page__hint">Searching…</p>}
            <ul className="add-member-modal__list direct-peer-list">
              {dmSearchQuery.trim().length >= 2
                ? dmSearchResults.map((peer) => (
                    <li key={peer.id}>
                      <button type="button" className="add-member-modal__item" onClick={() => void startDm(peer.id)}>
                        <span className="add-member-modal__label">{peer.displayName ?? peer.email}</span>
                        <span className="add-member-modal__meta">{peer.email}</span>
                      </button>
                    </li>
                  ))
                : peers.map((peer) => (
                    <li key={peer.id}>
                      <button type="button" className="add-member-modal__item" onClick={() => void startDm(peer.id)}>
                        <span className="add-member-modal__label">{peer.displayName ?? peer.email}</span>
                      </button>
                    </li>
                  ))}
              {dmSearchQuery.trim().length >= 2 &&
                !dmSearching &&
                dmSearchResults.length === 0 && (
                  <li className="add-member-modal__empty">No users found.</li>
                )}
              {dmSearchQuery.trim().length < 2 && peers.length === 0 && (
                <li className="add-member-modal__empty">Type at least 2 characters to search the platform.</li>
              )}
            </ul>
            <div className="chat-modal__actions">
              <button type="button" onClick={() => setShowNewDm(false)}>
                Cancel
              </button>
            </div>
          </div>
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
