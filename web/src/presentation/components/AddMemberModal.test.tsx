import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { AddMemberModal } from './AddMemberModal';

vi.mock('../../api/client', () => ({
  api: {
    searchUsers: vi.fn(),
    addMember: vi.fn(),
  },
}));

import { api } from '../../api/client';

afterEach(() => cleanup());

describe('AddMemberModal', () => {
  it('searches after typing and adds a selected user', async () => {
    vi.mocked(api.searchUsers).mockResolvedValue([
      { id: 'u2', email: 'friend@example.com', displayName: 'Friend' },
    ]);
    vi.mocked(api.addMember).mockResolvedValue({
      server_id: 's1',
      user_id: 'u2',
      role: 'member',
    });

    const onAdded = vi.fn();
    const onClose = vi.fn();
    render(<AddMemberModal serverId="s1" onAdded={onAdded} onClose={onClose} />);

    fireEvent.change(screen.getByLabelText('Search users'), {
      target: { value: 'friend' },
    });

    await waitFor(() => {
      expect(api.searchUsers).toHaveBeenCalledWith('friend', 's1');
    });

    fireEvent.click(screen.getByRole('button', { name: /Friend/ }));

    await waitFor(() => {
      expect(api.addMember).toHaveBeenCalledWith('s1', 'u2');
      expect(onAdded).toHaveBeenCalledOnce();
      expect(onClose).toHaveBeenCalledOnce();
    });
  });
});
