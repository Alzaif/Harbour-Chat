import { Outlet } from 'react-router-dom';
import { HarbourAppBar } from '../../components/HarbourAppBar';
import { BoardDock } from '../components/BoardDock';
import { PartyTopBar } from '../components/PartyTopBar';
import { TopNav } from './TopNav';
import { useSwipeNavigation } from './useSwipeNavigation';
import { UserSettingsProvider } from '../settings/UserSettingsContext';
import { PartySessionProvider } from '../voice/PartySessionContext';

const shellUrl = import.meta.env.VITE_HARBOUR_SHELL_URL?.trim() || window.location.origin;

export function BoardShell() {
  const swipe = useSwipeNavigation();

  return (
    <UserSettingsProvider>
      <PartySessionProvider>
        <div className="board-app">
          <HarbourAppBar homeUrl={shellUrl} appName="Board" center={<TopNav />} />
          <PartyTopBar />
          <main
            className="board-main"
            onPointerDown={swipe.onPointerDown}
            onPointerUp={swipe.onPointerUp}
            onPointerCancel={swipe.onPointerCancel}
          >
            <Outlet />
          </main>
          <BoardDock />
        </div>
      </PartySessionProvider>
    </UserSettingsProvider>
  );
}
