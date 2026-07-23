import { useState } from 'react';
import { Outlet, useParams } from 'react-router-dom';
import { AddMemberModal } from '../components/AddMemberModal';
import { MembersPanel } from '../components/MembersPanel';
import { ServerSidebar } from '../components/ServerSidebar';
import { ServerMembersProvider, useServerMembers } from './ServerMembersContext';

function ServersLayout({ hasThread }: { hasThread: boolean }) {
  const { serverId } = useParams<{ serverId?: string }>();
  const {
    members,
    presenceByUser,
    membersOpen,
    setMembersOpen,
    canManageMembers,
    reloadMembers,
  } = useServerMembers();
  const [showAddMember, setShowAddMember] = useState(false);

  const layoutClass = [
    'servers-layout',
    hasThread ? 'servers-layout--thread' : '',
    membersOpen ? 'servers-layout--members-open' : '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div className={layoutClass}>
      <ServerSidebar />
      <div className="servers-main">
        <Outlet />
      </div>
      {membersOpen && (
        <>
          <button
            type="button"
            className="groups-members-backdrop"
            aria-label="Close members"
            onClick={() => setMembersOpen(false)}
          />
          <MembersPanel
            members={members}
            presenceByUser={presenceByUser}
            open={membersOpen}
            onClose={() => setMembersOpen(false)}
            canAddMembers={canManageMembers}
            onAddMembers={() => setShowAddMember(true)}
          />
        </>
      )}
      {showAddMember && serverId && (
        <AddMemberModal
          serverId={serverId}
          onClose={() => setShowAddMember(false)}
          onAdded={() => reloadMembers()}
        />
      )}
    </div>
  );
}

export function ServersShell() {
  const { serverId, channelId } = useParams<{ serverId?: string; channelId?: string }>();

  return (
    <ServerMembersProvider serverId={serverId}>
      <ServersLayout hasThread={Boolean(channelId)} />
    </ServerMembersProvider>
  );
}
