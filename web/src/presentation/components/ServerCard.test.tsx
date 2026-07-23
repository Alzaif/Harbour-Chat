import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { Server } from '../../api/types';
import { ServerCard } from './ServerCard';

const sampleServer: Server = {
  id: 'srv-1',
  name: 'Gamer Server',
  description: 'Description text',
  icon_url: null,
  cardColor: '#ffffff',
  owner_user_id: 'dev-user',
  myRole: 'owner',
};

describe('ServerCard', () => {
  it('opens the server when the card is clicked', () => {
    const onOpen = vi.fn();
    render(<ServerCard server={sampleServer} onOpen={onOpen} />);

    fireEvent.click(screen.getByRole('button', { name: /Gamer Server/i }));

    expect(onOpen).toHaveBeenCalledOnce();
    expect(screen.getByText('Open →')).toBeInTheDocument();
  });

  it('shows admin actions when the user can manage the server', () => {
    const onEdit = vi.fn();
    const onDelete = vi.fn();
    render(
      <ServerCard
        server={sampleServer}
        canManage
        onOpen={vi.fn()}
        onEdit={onEdit}
        onDelete={onDelete}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Edit' }));
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }));

    expect(onEdit).toHaveBeenCalledOnce();
    expect(onDelete).toHaveBeenCalledOnce();
  });
});
