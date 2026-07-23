import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { CurrentUser, Message } from '../../api/types';
import { MessageBubble } from './MessageBubble';

afterEach(() => cleanup());

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

  it('hides the author/time meta and avatar initials when grouped', () => {
    const { container } = render(
      <MessageBubble message={baseMessage} currentUser={me} onToggleReaction={vi.fn()} grouped />,
    );
    expect(container.querySelector('.chat-message--grouped')).not.toBeNull();
    expect(screen.queryByText('Alice')).not.toBeInTheDocument();
    expect(container.querySelector('.chat-avatar--spacer')).not.toBeNull();
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });

  it('does not show quick reactions until the bubble is clicked', () => {
    render(
      <MessageBubble message={baseMessage} currentUser={me} onToggleReaction={vi.fn()} />,
    );
    expect(screen.queryByRole('menu', { name: 'Add reaction' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByText('Hello'));
    expect(screen.getByRole('menu', { name: 'Add reaction' })).toBeInTheDocument();
    expect(screen.getByRole('menuitem', { name: 'React 👍' })).toBeInTheDocument();
  });

  it('adds a reaction from the popup menu', () => {
    const onToggleReaction = vi.fn();
    render(
      <MessageBubble message={baseMessage} currentUser={me} onToggleReaction={onToggleReaction} />,
    );
    fireEvent.click(screen.getByText('Hello'));
    fireEvent.click(screen.getByRole('menuitem', { name: 'React ❤️' }));
    expect(onToggleReaction).toHaveBeenCalledWith('m1', '❤️');
    expect(screen.queryByRole('menu', { name: 'Add reaction' })).not.toBeInTheDocument();
  });

  it('only renders existing reaction chips when reactions exist', () => {
    const { container, unmount } = render(
      <MessageBubble message={baseMessage} currentUser={me} onToggleReaction={vi.fn()} />,
    );
    expect(container.querySelector('.chat-message__reactions')).toBeNull();
    unmount();

    const { container: reacted } = render(
      <MessageBubble
        message={{
          ...baseMessage,
          reactions: [{ emoji: '👍', count: 2, userIds: ['user-a', 'user-c'] }],
        }}
        currentUser={me}
        onToggleReaction={vi.fn()}
      />,
    );
    expect(reacted.querySelector('.chat-message__reactions')).not.toBeNull();
    expect(reacted.querySelector('.chat-reaction-btn')?.textContent).toBe('👍 2');
  });

  it('opens copy/reply/forward actions on right-click', () => {
    const onReply = vi.fn();
    const onForward = vi.fn();
    render(
      <MessageBubble
        message={baseMessage}
        currentUser={me}
        onToggleReaction={vi.fn()}
        onReply={onReply}
        onForward={onForward}
      />,
    );
    fireEvent.contextMenu(screen.getByText('Hello'));
    expect(screen.getByRole('menu', { name: 'Message actions' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('menuitem', { name: 'Reply' }));
    expect(onReply).toHaveBeenCalledWith(baseMessage);
  });

  it('renders a quoted parent above the reply body', () => {
    render(
      <MessageBubble
        message={{
          ...baseMessage,
          content: 'Sounds good',
          reply_to_message_id: 'm0',
          reply_to: {
            id: 'm0',
            author_user_id: 'user-c',
            author_display_name: 'Carol',
            content: 'Can we meet later?',
            deleted_at: null,
          },
        }}
        currentUser={me}
        onToggleReaction={vi.fn()}
      />,
    );
    expect(screen.getByText('Carol')).toBeInTheDocument();
    expect(screen.getByText('Can we meet later?')).toBeInTheDocument();
    expect(screen.getByText('Sounds good')).toBeInTheDocument();
  });
});
