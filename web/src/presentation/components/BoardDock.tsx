import { PartyBanner } from './PartyBanner';
import { UserDock } from './UserDock';
import { usePartySession } from '../voice/PartySessionContext';

export function BoardDock() {
  const party = usePartySession();

  return (
    <div className={`board-dock${party.inParty ? ' board-dock--in-party' : ''}`}>
      <PartyBanner />
      <UserDock />
    </div>
  );
}
