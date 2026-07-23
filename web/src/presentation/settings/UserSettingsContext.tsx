import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { api } from '../../api/client';
import type { UserSettings } from '../../api/types';

interface UserSettingsContextValue {
  settings: UserSettings;
  loading: boolean;
  updateSettings: (patch: Partial<UserSettings>) => Promise<void>;
}

const defaultSettings: UserSettings = { pushToTalk: false, pushToTalkKey: 'Space' };

const UserSettingsContext = createContext<UserSettingsContextValue | null>(null);

export function UserSettingsProvider({ children }: { children: React.ReactNode }) {
  const [settings, setSettings] = useState<UserSettings>(defaultSettings);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void api
      .getSettings()
      .then(setSettings)
      .catch(() => setSettings(defaultSettings))
      .finally(() => setLoading(false));
  }, []);

  const updateSettings = useCallback(async (patch: Partial<UserSettings>) => {
    const next = await api.updateSettings(patch);
    setSettings(next);
  }, []);

  const value = useMemo(
    () => ({ settings, loading, updateSettings }),
    [settings, loading, updateSettings],
  );

  return <UserSettingsContext.Provider value={value}>{children}</UserSettingsContext.Provider>;
}

export function useUserSettings() {
  const ctx = useContext(UserSettingsContext);
  if (!ctx) throw new Error('useUserSettings must be used within UserSettingsProvider');
  return ctx;
}
