import { useEffect, useState } from 'react';
import { api } from '../../api/client';
import type { Post, ShareTarget } from '../../api/types';

export interface SharePickerModalProps {
  post: Post;
  onClose: () => void;
  onShared: (label: string) => void;
}

export function SharePickerModal({ post, onClose, onShared }: SharePickerModalProps) {
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

  const shareTo = async (target: ShareTarget) => {
    setSharingId(target.channelId);
    setError(null);
    try {
      await api.shareBoardPost(post.id, target.channelId);
      onShared(target.label);
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Share failed');
    } finally {
      setSharingId(null);
    }
  };

  return (
    <div className="chat-modal-backdrop" role="dialog" aria-modal="true" aria-labelledby="share-title">
      <div className="chat-modal share-picker">
        <h2 id="share-title">Share post</h2>
        <p className="share-picker__hint">Discuss in a direct message or group channel.</p>
        {error && <p className="share-picker__error">{error}</p>}
        {loading ? (
          <p className="board-page__hint">Loading…</p>
        ) : targets.length === 0 ? (
          <p className="board-page__hint">No conversations yet. Open Direct or join a group first.</p>
        ) : (
          <ul className="share-picker__list">
            {targets.map((target) => (
              <li key={target.channelId}>
                <button
                  type="button"
                  className="share-picker__item"
                  disabled={sharingId === target.channelId}
                  onClick={() => void shareTo(target)}
                >
                  <span className="share-picker__label">{target.label}</span>
                  <span className="share-picker__kind">{target.kind === 'dm' ? 'Direct' : 'Group'}</span>
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
