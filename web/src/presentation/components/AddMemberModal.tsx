import { useEffect, useState } from 'react';
import { api } from '../../api/client';
import type { DmPeer } from '../../api/types';
import { ModalPortal } from './ModalPortal';

export interface AddMemberModalProps {
  serverId: string;
  onClose: () => void;
  onAdded: () => void;
}

export function AddMemberModal({ serverId, onClose, onAdded }: AddMemberModalProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<DmPeer[]>([]);
  const [searching, setSearching] = useState(false);
  const [addingId, setAddingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const trimmed = query.trim();
    if (trimmed.length < 2) {
      setResults([]);
      setError(null);
      return;
    }
    const handle = window.setTimeout(() => {
      setSearching(true);
      setError(null);
      void api
        .searchUsers(trimmed, serverId)
        .then(setResults)
        .catch((e) => {
          setResults([]);
          setError(e instanceof Error ? e.message : 'Search failed');
        })
        .finally(() => setSearching(false));
    }, 300);
    return () => window.clearTimeout(handle);
  }, [query, serverId]);

  const addUser = async (userId: string) => {
    setAddingId(userId);
    setError(null);
    try {
      await api.addMember(serverId, userId);
      onAdded();
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Could not add member');
    } finally {
      setAddingId(null);
    }
  };

  return (
    <ModalPortal>
      <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
      <div className="chat-modal add-member-modal">
        <h2>Add member</h2>
        <p className="add-member-modal__hint">
          Search Harbour users by name or email. They must have opened Board at least once.
        </p>
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search by name or email…"
          autoFocus
          aria-label="Search users"
        />
        {searching && <p className="board-page__hint">Searching…</p>}
        {error && (
          <p className="add-member-modal__error" role="alert">
            {error}
          </p>
        )}
        <ul className="add-member-modal__list">
          {query.trim().length >= 2 && !searching && results.length === 0 && !error && (
            <li className="add-member-modal__empty">No users found.</li>
          )}
          {results.map((user) => (
            <li key={user.id}>
              <button
                type="button"
                className="add-member-modal__item"
                disabled={addingId === user.id}
                onClick={() => void addUser(user.id)}
              >
                <span className="add-member-modal__label">
                  {user.displayName ?? user.email}
                </span>
                <span className="add-member-modal__meta">{user.email}</span>
              </button>
            </li>
          ))}
        </ul>
        <div className="chat-modal__actions">
          <button type="button" onClick={onClose}>
            Cancel
          </button>
        </div>
      </div>
      </div>
    </ModalPortal>
  );
}
