import { useEffect, useRef, useState } from 'react';
import type { CurrentUser, Message } from '../../api/types';
import { apiUrl } from '../../api/app-path';
import { formatRelativeTime } from '../utils/avatar';
import { Avatar } from './Avatar';

const QUICK_EMOJI = ['👍', '❤️', '😂', '🎉'];

export interface MessageBubbleProps {
  message: Message;
  currentUser: CurrentUser | null;
  onToggleReaction: (messageId: string, emoji: string) => void;
  /** When true, this message is merged under the previous one (same author, close in time). */
  grouped?: boolean;
  onReply?: (message: Message) => void;
  onForward?: (message: Message) => void;
}

export function MessageBubble({
  message,
  currentUser,
  onToggleReaction,
  grouped = false,
  onReply,
  onForward,
}: MessageBubbleProps) {
  const isOwn = currentUser != null && message.author_user_id === currentUser.id;
  const displayName = message.author_display_name ?? message.author_user_id;
  const bodyRef = useRef<HTMLDivElement>(null);
  const [reactionMenuOpen, setReactionMenuOpen] = useState(false);
  const [contextMenuOpen, setContextMenuOpen] = useState(false);

  useEffect(() => {
    if (!reactionMenuOpen && !contextMenuOpen) return;
    const onPointerDown = (event: PointerEvent) => {
      if (bodyRef.current?.contains(event.target as Node)) return;
      setReactionMenuOpen(false);
      setContextMenuOpen(false);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setReactionMenuOpen(false);
        setContextMenuOpen(false);
      }
    };
    document.addEventListener('pointerdown', onPointerDown);
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('pointerdown', onPointerDown);
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [reactionMenuOpen, contextMenuOpen]);

  const avatar = grouped ? (
    <span className="chat-avatar chat-avatar--spacer" aria-hidden />
  ) : (
    <Avatar userId={message.author_user_id} name={displayName} />
  );

  const existingReactions = message.reactions?.filter((r) => r.count > 0) ?? [];

  const handleBubbleClick = (event: React.MouseEvent) => {
    if (message.deleted_at) return;
    const target = event.target as HTMLElement;
    if (target.closest('a, button')) return;
    setContextMenuOpen(false);
    setReactionMenuOpen((open) => !open);
  };

  const handleContextMenu = (event: React.MouseEvent) => {
    if (message.deleted_at) return;
    event.preventDefault();
    setReactionMenuOpen(false);
    setContextMenuOpen(true);
  };

  const pickReaction = (emoji: string) => {
    onToggleReaction(message.id, emoji);
    setReactionMenuOpen(false);
  };

  const copyMessage = async () => {
    setContextMenuOpen(false);
    if (!message.content) return;
    try {
      await navigator.clipboard.writeText(message.content);
    } catch {
      /* clipboard may be denied */
    }
  };

  const replyQuote = message.reply_to;
  const quoteAuthor =
    replyQuote?.author_display_name ?? replyQuote?.author_user_id ?? 'Unknown';
  const quoteText = replyQuote?.deleted_at
    ? 'Original message was deleted'
    : (replyQuote?.content ?? '');

  return (
    <article
      className={`chat-message${isOwn ? ' chat-message--own' : ''}${grouped ? ' chat-message--grouped' : ''}`}
      aria-label={`Message from ${displayName}`}
    >
      {!isOwn && avatar}
      <div className="chat-message__body" ref={bodyRef}>
        {!grouped && (
          <div className="chat-message__meta">
            <span className="chat-message__author">{isOwn ? 'You' : displayName}</span>
            <time className="chat-message__time" dateTime={message.created_at}>
              {formatRelativeTime(message.created_at)}
            </time>
          </div>
        )}
        <div className="chat-message__bubble-wrap">
          {!message.deleted_at && reactionMenuOpen && (
            <div className="chat-reaction-menu" role="menu" aria-label="Add reaction">
              {QUICK_EMOJI.map((emoji) => (
                <button
                  key={emoji}
                  type="button"
                  role="menuitem"
                  className="chat-reaction-menu__btn"
                  aria-label={`React ${emoji}`}
                  onClick={() => pickReaction(emoji)}
                >
                  {emoji}
                </button>
              ))}
            </div>
          )}
          {!message.deleted_at && contextMenuOpen && (
            <div className="chat-context-menu" role="menu" aria-label="Message actions">
              <button type="button" role="menuitem" className="chat-context-menu__item" onClick={() => void copyMessage()}>
                Copy message
              </button>
              {onReply ? (
                <button
                  type="button"
                  role="menuitem"
                  className="chat-context-menu__item"
                  onClick={() => {
                    setContextMenuOpen(false);
                    onReply(message);
                  }}
                >
                  Reply
                </button>
              ) : null}
              {onForward ? (
                <button
                  type="button"
                  role="menuitem"
                  className="chat-context-menu__item"
                  onClick={() => {
                    setContextMenuOpen(false);
                    onForward(message);
                  }}
                >
                  Forward
                </button>
              ) : null}
            </div>
          )}
          <div
            className={`chat-message__bubble${isOwn ? ' chat-message__bubble--own' : ''}${reactionMenuOpen || contextMenuOpen ? ' chat-message__bubble--menu-open' : ''}${message.deleted_at ? '' : ' chat-message__bubble--clickable'}`}
            onClick={handleBubbleClick}
            onContextMenu={handleContextMenu}
            onKeyDown={(event) => {
              if (message.deleted_at) return;
              if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
                setContextMenuOpen(false);
                setReactionMenuOpen((open) => !open);
              }
              if (event.key === 'Escape') {
                setReactionMenuOpen(false);
                setContextMenuOpen(false);
              }
            }}
            role={message.deleted_at ? undefined : 'button'}
            tabIndex={message.deleted_at ? undefined : 0}
            aria-haspopup={message.deleted_at ? undefined : 'menu'}
            aria-expanded={message.deleted_at ? undefined : reactionMenuOpen || contextMenuOpen}
          >
            {message.deleted_at ? (
              <em>(deleted)</em>
            ) : (
              <>
                {replyQuote ? (
                  <blockquote className="chat-message__quote">
                    <span className="chat-message__quote-author">{quoteAuthor}</span>
                    <span className="chat-message__quote-text">{quoteText}</span>
                  </blockquote>
                ) : null}
                {message.content && <p className="chat-message__text">{message.content}</p>}
                {message.attachment && message.attachment.mime_type?.startsWith('image/') && (
                  <img
                    className="chat-message__image"
                    src={apiUrl(`/api/attachments/${message.attachment.id}`)}
                    alt={message.attachment.filename}
                  />
                )}
                {message.attachment && !message.attachment.mime_type?.startsWith('image/') && (
                  <a
                    className="chat-message__file"
                    href={apiUrl(`/api/attachments/${message.attachment.id}`)}
                    download={message.attachment.filename}
                  >
                    {message.attachment.filename}
                  </a>
                )}
              </>
            )}
          </div>
        </div>
        {existingReactions.length > 0 && (
          <div className="chat-message__reactions">
            {existingReactions.map((r) => (
              <button
                key={r.emoji}
                type="button"
                className={`chat-reaction-btn${
                  currentUser && r.userIds.includes(currentUser.id) ? ' is-active' : ''
                }`}
                onClick={() => onToggleReaction(message.id, r.emoji)}
              >
                {r.emoji} {r.count}
              </button>
            ))}
          </div>
        )}
      </div>
      {isOwn && avatar}
    </article>
  );
}
