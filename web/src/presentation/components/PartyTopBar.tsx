import { usePartySession } from '../voice/PartySessionContext';

export function PartyTopBar() {
  const party = usePartySession();

  if (!party.inParty || !party.origin) return null;

  return (
    <div className="party-topbar" role="region" aria-label="Active party">
      <div className="party-topbar__info">
        <span className="party-topbar__dot" aria-hidden />
        <span className="party-topbar__origin">
          {party.origin.serverName} · {party.origin.channelName}
        </span>
      </div>
      <div className="party-topbar__controls">
        <button type="button" onClick={() => void party.toggleMute()}>
          {party.voiceMuted ? 'Unmute' : 'Mute'}
        </button>
        <button type="button" onClick={() => void party.toggleDeafen()}>
          {party.voiceDeafened ? 'Undeafen' : 'Deafen'}
        </button>
        <button
          type="button"
          className="party-topbar__disconnect"
          onClick={() => void party.leaveParty()}
        >
          Disconnect
        </button>
      </div>
    </div>
  );
}
