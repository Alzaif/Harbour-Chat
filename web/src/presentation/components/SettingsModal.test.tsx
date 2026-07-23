import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { UserSettingsProvider } from '../settings/UserSettingsContext';
import { SettingsModal } from './SettingsModal';

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

vi.mock('../../api/client', () => ({
  api: {
    getSettings: vi.fn().mockResolvedValue({ pushToTalk: false, pushToTalkKey: 'Space' }),
    updateSettings: vi.fn().mockResolvedValue({ pushToTalk: true }),
    me: vi
      .fn()
      .mockResolvedValue({ id: 'u1', email: 'a@b', displayName: 'Alice', avatarUpdatedAt: null }),
    uploadAvatar: vi
      .fn()
      .mockResolvedValue({ mimeType: 'image/png', sizeBytes: 10, avatarUpdatedAt: 42 }),
  },
}));

describe('SettingsModal', () => {
  it('toggles push to talk and persists the setting', async () => {
    const { api } = await import('../../api/client');
    render(
      <UserSettingsProvider>
        <SettingsModal onClose={vi.fn()} />
      </UserSettingsProvider>,
    );

    await waitFor(() => {
      expect(screen.getByRole('checkbox')).not.toBeChecked();
    });

    fireEvent.click(screen.getByRole('checkbox'));

    await waitFor(() => {
      expect(api.updateSettings).toHaveBeenCalledWith({ pushToTalk: true });
    });
  });

  it('captures a non-preset key such as an arrow key', async () => {
    const { api } = await import('../../api/client');
    render(
      <UserSettingsProvider>
        <SettingsModal onClose={vi.fn()} />
      </UserSettingsProvider>,
    );

    fireEvent.click(await screen.findByRole('button', { name: 'Set key' }));
    fireEvent.keyDown(window, { code: 'ArrowUp' });

    await waitFor(() => {
      expect(api.updateSettings).toHaveBeenCalledWith({ pushToTalkKey: 'ArrowUp' });
    });
  });

  it('uploads a selected avatar and notifies the parent', async () => {
    const { api } = await import('../../api/client');
    const onProfileUpdated = vi.fn();
    render(
      <UserSettingsProvider>
        <SettingsModal onClose={vi.fn()} onProfileUpdated={onProfileUpdated} />
      </UserSettingsProvider>,
    );

    const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
    const file = new File(['x'], 'me.png', { type: 'image/png' });
    fireEvent.change(fileInput, { target: { files: [file] } });

    await waitFor(() => {
      expect(api.uploadAvatar).toHaveBeenCalledWith(file);
      expect(onProfileUpdated).toHaveBeenCalled();
    });
  });

  it('cancels key capture when Escape is pressed', async () => {
    const { api } = await import('../../api/client');
    render(
      <UserSettingsProvider>
        <SettingsModal onClose={vi.fn()} />
      </UserSettingsProvider>,
    );

    fireEvent.click(await screen.findByRole('button', { name: 'Set key' }));
    fireEvent.keyDown(window, { code: 'Escape' });

    expect(await screen.findByRole('button', { name: 'Set key' })).toBeInTheDocument();
    expect(api.updateSettings).not.toHaveBeenCalledWith(
      expect.objectContaining({ pushToTalkKey: expect.anything() }),
    );
  });
});
