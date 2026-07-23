import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { PartyMenu } from './PartyMenu';

afterEach(() => cleanup());

describe('PartyMenu', () => {
  it('opens menu and calls Start Party', () => {
    const onStartParty = vi.fn();
    render(
      <PartyMenu
        voiceChannels={[]}
        inVoiceChannelId={null}
        onStartParty={onStartParty}
        onJoinParty={vi.fn()}
        onCreateParty={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Party' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Start Party' }));
    expect(onStartParty).toHaveBeenCalledOnce();
  });

  it('lists voice channels under Join Party', () => {
    const onJoinParty = vi.fn();
    render(
      <PartyMenu
        voiceChannels={[{ id: 'v1', name: 'Lounge', participantCount: 2 }]}
        inVoiceChannelId={null}
        onStartParty={vi.fn()}
        onJoinParty={onJoinParty}
        onCreateParty={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Party' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Join Party' }));
    fireEvent.click(screen.getByRole('menuitem', { name: /Lounge/ }));
    expect(onJoinParty).toHaveBeenCalledWith('v1');
  });

  it('shows Leave Party when already in voice', () => {
    const onLeaveParty = vi.fn();
    render(
      <PartyMenu
        voiceChannels={[{ id: 'v1', name: 'Lounge' }]}
        inVoiceChannelId="v1"
        onStartParty={vi.fn()}
        onJoinParty={vi.fn()}
        onCreateParty={vi.fn()}
        onLeaveParty={onLeaveParty}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Party' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Leave Party' }));
    expect(onLeaveParty).toHaveBeenCalledOnce();
  });
});
