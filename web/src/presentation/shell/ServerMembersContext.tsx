import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { api } from '../../api/client';
import type { CurrentUser, Member, PresenceStatus } from '../../api/types';

interface ServerMembersContextValue {
  members: Member[];
  presenceByUser: Record<string, PresenceStatus>;
  membersOpen: boolean;
  setMembersOpen: (open: boolean) => void;
  toggleMembers: () => void;
  canManageMembers: boolean;
  reloadMembers: () => void;
}

const ServerMembersContext = createContext<ServerMembersContextValue | null>(null);

export function ServerMembersProvider({
  serverId,
  children,
}: {
  serverId?: string;
  children: ReactNode;
}) {
  const [members, setMembers] = useState<Member[]>([]);
  const [presenceByUser, setPresenceByUser] = useState<Record<string, PresenceStatus>>({});
  const [currentUser, setCurrentUser] = useState<CurrentUser | null>(null);
  const [membersOpen, setMembersOpen] = useState(false);

  const reloadMembers = useCallback(() => {
    if (!serverId) return;
    void api
      .listMembers(serverId)
      .then(setMembers)
      .catch(() => {});
  }, [serverId]);

  useEffect(() => {
    setMembersOpen(false);
    if (!serverId) {
      setMembers([]);
      setPresenceByUser({});
      return;
    }
    let cancelled = false;
    void (async () => {
      try {
        const [me, list, presence] = await Promise.all([
          api.me(),
          api.listMembers(serverId),
          api.listPresence(serverId),
        ]);
        if (cancelled) return;
        setCurrentUser(me);
        setMembers(list);
        const mapped: Record<string, PresenceStatus> = {};
        for (const entry of presence) mapped[entry.user_id] = entry.status;
        setPresenceByUser(mapped);
      } catch {
        /* presence/members are non-critical for rendering the thread */
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [serverId]);

  const canManageMembers = useMemo(() => {
    if (!currentUser) return false;
    const membership = members.find((member) => member.user_id === currentUser.id);
    return membership?.role === 'owner' || membership?.role === 'admin';
  }, [currentUser, members]);

  const value = useMemo<ServerMembersContextValue>(
    () => ({
      members,
      presenceByUser,
      membersOpen,
      setMembersOpen,
      toggleMembers: () => setMembersOpen((open) => !open),
      canManageMembers,
      reloadMembers,
    }),
    [members, presenceByUser, membersOpen, canManageMembers, reloadMembers],
  );

  return <ServerMembersContext.Provider value={value}>{children}</ServerMembersContext.Provider>;
}

export function useServerMembers() {
  const ctx = useContext(ServerMembersContext);
  if (!ctx) {
    throw new Error('useServerMembers must be used within a ServerMembersProvider');
  }
  return ctx;
}
