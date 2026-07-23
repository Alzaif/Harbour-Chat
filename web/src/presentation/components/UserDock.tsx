import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api/client';
import type { CurrentUser } from '../../api/types';
import { Avatar } from './Avatar';
import { SettingsModal } from './SettingsModal';

export function UserDock() {
  const [user, setUser] = useState<CurrentUser | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);

  const reload = useCallback(() => {
    void api
      .me()
      .then(setUser)
      .catch(() => setUser(null));
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  const label = user?.displayName ?? user?.email ?? 'User';

  return (
    <>
      <div className="user-dock">
        <Avatar
          className="user-dock__avatar"
          userId={user?.id}
          name={label}
          version={user?.avatarUpdatedAt}
        />
        <span className="user-dock__name">{label}</span>
        <button
          type="button"
          className="user-dock__settings"
          aria-label="Open settings"
          onClick={() => setSettingsOpen(true)}
        >
          ⚙
        </button>
      </div>
      {settingsOpen && (
        <SettingsModal onClose={() => setSettingsOpen(false)} onProfileUpdated={reload} />
      )}
    </>
  );
}
