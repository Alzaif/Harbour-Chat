import { NavLink } from 'react-router-dom';
import { BOARD_TABS, TAB_DISPLAY } from './board-routes';

export function TopNav() {
  return (
    <nav className="harbour-chrome__nav" aria-label="Board sections">
      {BOARD_TABS.map((tab) => (
        <NavLink
          key={tab}
          to={tab === 'feed' ? '/feed' : `/${tab}`}
          className={({ isActive }) =>
            `harbour-chrome__nav-tab${isActive ? ' harbour-chrome__nav-tab--active' : ''}`
          }
          end={tab !== 'servers'}
        >
          {TAB_DISPLAY[tab]}
        </NavLink>
      ))}
    </nav>
  );
}
