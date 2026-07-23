import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it } from 'vitest';
import { HarbourAppBar } from '../../components/HarbourAppBar';
import { TopNav } from './TopNav';

describe('Board header navigation', () => {
  it('renders the section tabs integrated inside the app header', () => {
    render(
      <MemoryRouter initialEntries={['/servers']}>
        <HarbourAppBar homeUrl="https://harbour.local" appName="Board" center={<TopNav />} />
      </MemoryRouter>,
    );

    const header = document.querySelector('.harbour-chrome');
    expect(header).not.toBeNull();

    const nav = header?.querySelector('.harbour-chrome__nav');
    expect(nav).not.toBeNull();

    // The old second-row nav bar should no longer exist.
    expect(document.querySelector('.board-top-nav')).toBeNull();

    for (const label of ['Direct', 'Servers', 'Board']) {
      expect(screen.getByRole('link', { name: label })).toBeInTheDocument();
    }
  });

  it('marks the active section based on the current route', () => {
    render(
      <MemoryRouter initialEntries={['/servers']}>
        <HarbourAppBar homeUrl="https://harbour.local" appName="Board" center={<TopNav />} />
      </MemoryRouter>,
    );

    const active = document.querySelector('.harbour-chrome__nav-tab--active');
    expect(active?.textContent).toBe('Servers');
  });
});
