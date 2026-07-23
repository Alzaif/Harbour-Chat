import type { Member, PresenceStatus } from '../../api/types';
import { Avatar } from './Avatar';

export interface MembersPanelProps {
  members: Member[];
  presenceByUser: Record<string, PresenceStatus>;
  open: boolean;
  onClose: () => void;
  canAddMembers?: boolean;
  onAddMembers?: () => void;
}

export function MembersPanel({
  members,
  presenceByUser,
  open,
  onClose,
  canAddMembers = false,
  onAddMembers,
}: MembersPanelProps) {
  if (!open) return null;

  return (
    <aside className="chat-members" aria-label="Members">
      <div className="chat-members__header">
        <h2>Members — {members.length}</h2>
        <div className="chat-members__header-actions">
          {canAddMembers && onAddMembers && (
            <button type="button" className="chat-members__add" onClick={onAddMembers}>
              Add
            </button>
          )}
          <button type="button" className="chat-members__close" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
      <ul className="chat-members__list">
        {members.map((m) => {
          const name = m.display_name ?? m.user_id;
          return (
            <li key={m.user_id}>
              <Avatar userId={m.user_id} name={name} />
              <span className="chat-members__name">{name}</span>
              <span
                className={`chat-presence-dot chat-presence-dot--${presenceByUser[m.user_id] ?? 'offline'}`}
                title={presenceByUser[m.user_id] ?? 'offline'}
              />
              <span className="chat-members__role">{m.role}</span>
            </li>
          );
        })}
      </ul>
    </aside>
  );
}
