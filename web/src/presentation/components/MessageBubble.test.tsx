import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { CurrentUser, Message } from '../../api/types';
import { MessageBubble } from './MessageBubble';

const baseMessage: Message = {
  id: 'm1',
  channel_id: 'c1',
  author_user_id: 'user-a',
  author_display_name: 'Alice',
  content: 'Hello',
  created_at: new Date().toISOString(),
  edited_at: null,
  deleted_at: null,
  reactions: [],
};

const me: CurrentUser = { id: 'user-b', email: 'b@test', displayName: 'Bob' };

describe('MessageBubble', () => {
  it('marks other messages without own class', () => {
    const { container } = render(
      <MessageBubble message={baseMessage} currentUser={me} onToggleReaction={vi.fn()} />,
    );
    expect(container.querySelector('.chat-message--own')).toBeNull();
    expect(screen.getByText('Alice')).toBeInTheDocument();
  });

  it('marks own messages with chat-message--own', () => {
    const own: Message = { ...baseMessage, author_user_id: 'user-b' };
    const { container } = render(
      <MessageBubble message={own} currentUser={me} onToggleReaction={vi.fn()} />,
    );
    expect(container.querySelector('.chat-message--own')).not.toBeNull();
    expect(screen.getByText('You')).toBeInTheDocument();
  });
});
