import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { VoteControls } from './VoteControls';

afterEach(() => cleanup());

describe('VoteControls', () => {
  it('marks active upvote and reports clicks', () => {
    const onVote = vi.fn();
    render(<VoteControls score={5} myVote={1} onVote={onVote} />);
    const up = screen.getByRole('button', { name: 'Upvote' });
    expect(up).toHaveAttribute('aria-pressed', 'true');
    screen.getByRole('button', { name: 'Downvote' }).click();
    expect(onVote).toHaveBeenCalledWith(-1);
  });
});
