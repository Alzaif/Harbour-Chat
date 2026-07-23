import { cleanup, render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Post } from '../../api/types';
import { StoryRow } from './StoryRow';

afterEach(() => cleanup());

const post: Post = {
  id: 'p1',
  authorUserId: 'u1',
  authorDisplayName: 'Alice',
  title: 'Harbour opens newspaper feed',
  body: 'Body text for the story',
  linkUrl: 'https://example.com',
  previewTitle: null,
  previewDescription: null,
  previewImageUrl: null,
  previewSiteName: 'example.com',
  upvotes: 3,
  downvotes: 1,
  score: 2,
  commentCount: 4,
  myVote: 0,
  createdAt: '2026-06-15T12:00:00Z',
  updatedAt: '2026-06-15T12:00:00Z',
};

describe('StoryRow', () => {
  it('renders headline, score, and comment count', () => {
    render(
      <MemoryRouter>
        <StoryRow post={post} onVote={() => {}} />
      </MemoryRouter>,
    );
    expect(screen.getByText('Harbour opens newspaper feed')).toBeInTheDocument();
    expect(screen.getByLabelText('Score 2')).toHaveTextContent('2');
    expect(screen.getByText('4 comments')).toBeInTheDocument();
  });

  it('calls onVote for upvote', () => {
    const onVote = vi.fn();
    render(
      <MemoryRouter>
        <StoryRow post={post} onVote={onVote} />
      </MemoryRouter>,
    );
    screen.getByRole('button', { name: 'Upvote' }).click();
    expect(onVote).toHaveBeenCalledWith('p1', 1);
  });
});
