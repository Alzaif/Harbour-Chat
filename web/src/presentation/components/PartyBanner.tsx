import { usePartySession } from '../voice/PartySessionContext';
import { useUserSettings } from '../settings/UserSettingsContext';
import { formatKeyCode } from '../utils/push-to-talk-key';

export function PartyBanner() {
  const party = usePartySession();
  const { settings } = useUserSettings();

  if (!party.inParty || !party.origin) return null;

  const pttHint = settings.pushToTalk
    ? `Hold ${formatKeyCode(settings.pushToTalkKey || 'Space')} to talk`
    : null;

  return (
    <div className="party-banner" role="region" aria-label="Active party">
      <div className="party-banner__meta">
        <span className="party-banner__label">In Party</span>
        <span className="party-banner__origin">
          {party.origin.serverName} · {party.origin.channelName}
        </span>
        {pttHint && <span className="party-banner__ptt">{pttHint}</span>}
        {party.error && <span className="party-banner__error">{party.error}</span>}
      </div>
      <div className="party-banner__controls">
        <button type="button" onClick={() => void party.toggleMute()}>
          {party.voiceMuted ? 'Unmute' : 'Mute'}
        </button>
        <button type="button" onClick={() => void party.toggleDeafen()}>
          {party.voiceDeafened ? 'Undeafen' : 'Deafen'}
        </button>
        <button type="button" className="party-banner__disconnect" onClick={() => void party.leaveParty()}>
          Disconnect
        </button>
      </div>
    </div>
  );
}
