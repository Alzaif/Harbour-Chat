import { useEffect, useRef, useState } from 'react';
import { api } from '../../api/client';
import type { CurrentUser } from '../../api/types';
import { useUserSettings } from '../settings/UserSettingsContext';
import { Avatar } from './Avatar';
import { ModalPortal } from './ModalPortal';
import { PRESET_KEYS, formatKeyCode, isAllowedPushToTalkKey } from '../utils/push-to-talk-key';

export interface SettingsModalProps {
  onClose: () => void;
  /** Called after the profile (e.g. avatar) changes so parents can refresh. */
  onProfileUpdated?: () => void;
}

export function SettingsModal({ onClose, onProfileUpdated }: SettingsModalProps) {
  const { settings, updateSettings } = useUserSettings();
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [capturingKey, setCapturingKey] = useState(false);
  const [me, setMe] = useState<CurrentUser | null>(null);
  const [uploadingAvatar, setUploadingAvatar] = useState(false);
  const avatarInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    void api
      .me()
      .then(setMe)
      .catch(() => setMe(null));
  }, []);

  const onAvatarSelected = async (file: File | null) => {
    if (!file) return;
    setUploadingAvatar(true);
    setError(null);
    try {
      const result = await api.uploadAvatar(file);
      setMe((prev) => (prev ? { ...prev, avatarUpdatedAt: result.avatarUpdatedAt } : prev));
      onProfileUpdated?.();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Could not upload photo');
    } finally {
      setUploadingAvatar(false);
      if (avatarInputRef.current) avatarInputRef.current.value = '';
    }
  };

  useEffect(() => {
    if (!capturingKey) return;
    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      if (event.code === 'Escape') {
        setCapturingKey(false);
        return;
      }
      if (!isAllowedPushToTalkKey(event.code)) {
        setError('That key can\u2019t be used. Pick another key.');
        setCapturingKey(false);
        return;
      }
      setSaving(true);
      setError(null);
      void updateSettings({ pushToTalkKey: event.code })
        .catch((e) => setError(e instanceof Error ? e.message : 'Could not save key'))
        .finally(() => {
          setSaving(false);
          setCapturingKey(false);
        });
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [capturingKey, updateSettings]);

  const togglePushToTalk = async () => {
    setSaving(true);
    setError(null);
    try {
      await updateSettings({ pushToTalk: !settings.pushToTalk });
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Could not save settings');
    } finally {
      setSaving(false);
    }
  };

  const setPresetKey = async (keyCode: string) => {
    setSaving(true);
    setError(null);
    try {
      await updateSettings({ pushToTalkKey: keyCode });
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Could not save key');
    } finally {
      setSaving(false);
    }
  };

  const currentKeyLabel = formatKeyCode(settings.pushToTalkKey || 'Space');

  return (
    <ModalPortal>
      <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
        <div className="chat-modal settings-modal">
          <h2>Settings</h2>
          <div className="settings-modal__field">
            <span>Profile photo</span>
            <div className="settings-modal__avatar-row">
              <Avatar
                className="settings-modal__avatar"
                userId={me?.id}
                name={me?.displayName ?? me?.email ?? 'You'}
                version={me?.avatarUpdatedAt}
              />
              <input
                ref={avatarInputRef}
                type="file"
                accept="image/png,image/jpeg,image/gif,image/webp"
                hidden
                onChange={(e) => void onAvatarSelected(e.target.files?.[0] ?? null)}
              />
              <button
                type="button"
                disabled={uploadingAvatar}
                onClick={() => avatarInputRef.current?.click()}
              >
                {uploadingAvatar ? 'Uploading…' : 'Upload photo'}
              </button>
            </div>
            <span className="settings-modal__hint">PNG, JPEG, GIF or WebP.</span>
          </div>
          <label className="settings-modal__row">
            <span>
              Push to talk
              <span className="settings-modal__hint">
                Hold your chosen key while in a Party to speak
              </span>
            </span>
            <input
              type="checkbox"
              checked={settings.pushToTalk}
              disabled={saving}
              onChange={() => void togglePushToTalk()}
            />
          </label>
          <label className="settings-modal__field">
            <span>Push to talk key</span>
            <div className="settings-modal__key-row">
              <select
                value={PRESET_KEYS.includes(settings.pushToTalkKey as (typeof PRESET_KEYS)[number]) ? settings.pushToTalkKey : 'custom'}
                disabled={saving || capturingKey}
                onChange={(e) => {
                  const value = e.target.value;
                  if (value !== 'custom') void setPresetKey(value);
                }}
              >
                {PRESET_KEYS.map((key) => (
                  <option key={key} value={key}>
                    {formatKeyCode(key)}
                  </option>
                ))}
                {!PRESET_KEYS.includes(settings.pushToTalkKey as (typeof PRESET_KEYS)[number]) && (
                  <option value="custom">{currentKeyLabel}</option>
                )}
                <option value="custom">Custom…</option>
              </select>
              <button
                type="button"
                disabled={saving}
                onClick={() => {
                  setError(null);
                  setCapturingKey(true);
                }}
              >
                {capturingKey ? 'Press any key… (Esc to cancel)' : 'Set key'}
              </button>
            </div>
            <span className="settings-modal__hint">Current: {currentKeyLabel}</span>
          </label>
          {error && (
            <p className="add-member-modal__error" role="alert">
              {error}
            </p>
          )}
          <div className="chat-modal__actions">
            <button type="button" onClick={onClose}>
              Close
            </button>
          </div>
        </div>
      </div>
    </ModalPortal>
  );
}
