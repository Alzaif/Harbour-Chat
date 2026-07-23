import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { PostCard } from './PostCard';
import type { Post } from '../../api/types';

afterEach(() => cleanup());

const post: Post = {
  id: 'p1',
  authorUserId: 'u1',
  authorDisplayName: 'Alice',
  title: 'Test headline',
  body: 'Body text for the card',
  linkUrl: 'https://example.com',
  previewTitle: null,
  previewDescription: null,
  previewImageUrl: null,
  previewSiteName: 'example.com',
  upvotes: 0,
  downvotes: 0,
  score: 0,
  commentCount: 0,
  myVote: 0,
  createdAt: '2026-06-15T12:00:00Z',
  updatedAt: '2026-06-15T12:00:00Z',
};

describe('PostCard', () => {
  it('renders title and calls onShare', async () => {
    const onShare = vi.fn();
    render(<PostCard post={post} onToggleExpand={() => {}} onShare={onShare} />);
    expect(screen.getByText('Test headline')).toBeInTheDocument();
    screen.getByRole('button', { name: 'Share' }).click();
    expect(onShare).toHaveBeenCalledOnce();
  });

  it('links preview to external URL', () => {
    render(<PostCard post={post} onToggleExpand={() => {}} />);
    const link = screen.getByRole('link', { name: 'example.com' });
    expect(link).toHaveAttribute('href', 'https://example.com');
  });
});
