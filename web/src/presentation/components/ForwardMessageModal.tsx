import { useEffect, useState } from 'react';
import { api } from '../../api/client';
import type { Message, ShareTarget } from '../../api/types';

export interface ForwardMessageModalProps {
  message: Message;
  onClose: () => void;
  onForwarded: () => void;
}

function formatForwardBody(message: Message): string {
  const author = message.author_display_name ?? message.author_user_id;
  const body = message.content.trim() || '(attachment)';
  return `Forwarded from ${author}:\n${body}`;
}

export function ForwardMessageModal({ message, onClose, onForwarded }: ForwardMessageModalProps) {
  const [targets, setTargets] = useState<ShareTarget[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [sharingId, setSharingId] = useState<string | null>(null);

  useEffect(() => {
    void api
      .listShareTargets()
      .then(setTargets)
      .catch((e) => setError(e instanceof Error ? e.message : 'Failed to load destinations'))
      .finally(() => setLoading(false));
  }, []);

  const forwardTo = async (channelId: string) => {
    setSharingId(channelId);
    setError(null);
    try {
      await api.sendMessage(channelId, formatForwardBody(message));
      onForwarded();
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Forward failed');
    } finally {
      setSharingId(null);
    }
  };

  return (
    <div className="chat-modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="forward-title">
      <div className="chat-modal share-picker">
        <h2 id="forward-title">Forward message</h2>
        <p className="share-picker__hint">Send a copy to a direct message or server channel.</p>
        {error && <p className="share-picker__error">{error}</p>}
        {loading ? (
          <p className="board-page__hint">Loading…</p>
        ) : targets.length === 0 ? (
          <p className="board-page__hint">No conversations yet. Open Direct or join a server first.</p>
        ) : (
          <ul className="share-picker__list">
            {targets.map((target) => (
              <li key={target.channelId}>
                <button
                  type="button"
                  className="share-picker__item"
                  disabled={sharingId === target.channelId}
                  onClick={() => void forwardTo(target.channelId)}
                >
                  <span className="share-picker__label">{target.label}</span>
                  <span className="share-picker__kind">{target.kind === 'dm' ? 'Direct' : 'Server'}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
        <div className="chat-modal__actions">
          <button type="button" onClick={onClose}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
