import type { CurrentUser, Message } from '../../api/types';
import { colorFromName, formatRelativeTime, initialsFromName } from '../utils/avatar';

const QUICK_EMOJI = ['👍', '❤️', '😂', '🎉'];

export interface MessageBubbleProps {
  message: Message;
  currentUser: CurrentUser | null;
  onToggleReaction: (messageId: string, emoji: string) => void;
}

export function MessageBubble({ message, currentUser, onToggleReaction }: MessageBubbleProps) {
  const isOwn = currentUser != null && message.author_user_id === currentUser.id;
  const displayName = message.author_display_name ?? message.author_user_id;
  const avatarColor = colorFromName(displayName);

  return (
    <article
      className={`chat-message${isOwn ? ' chat-message--own' : ''}`}
      aria-label={`Message from ${displayName}`}
    >
      {!isOwn && (
        <span className="chat-avatar" aria-hidden style={{ background: avatarColor }}>
          {initialsFromName(displayName)}
        </span>
      )}
      <div className="chat-message__body">
        <div className="chat-message__meta">
          <span className="chat-message__author">{isOwn ? 'You' : displayName}</span>
          <time className="chat-message__time" dateTime={message.created_at}>
            {formatRelativeTime(message.created_at)}
          </time>
        </div>
        <div className={`chat-message__bubble${isOwn ? ' chat-message__bubble--own' : ''}`}>
          {message.deleted_at ? (
            <em>(deleted)</em>
          ) : (
            <>
              {message.content && <p className="chat-message__text">{message.content}</p>}
              {message.attachment && message.attachment.mime_type?.startsWith('image/') && (
                <img
                  className="chat-message__image"
                  src={`/api/attachments/${message.attachment.id}`}
                  alt={message.attachment.filename}
                />
              )}
              {message.attachment && !message.attachment.mime_type?.startsWith('image/') && (
                <a
                  className="chat-message__file"
                  href={`/api/attachments/${message.attachment.id}`}
                  download={message.attachment.filename}
                >
                  {message.attachment.filename}
                </a>
              )}
            </>
          )}
        </div>
        {!message.deleted_at && (
          <div className="chat-message__reactions">
            {message.reactions?.map((r) => (
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
            <span className="chat-reaction-quick">
              {QUICK_EMOJI.map((emoji) => (
                <button
                  key={emoji}
                  type="button"
                  className="chat-reaction-add"
                  aria-label={`React ${emoji}`}
                  onClick={() => onToggleReaction(message.id, emoji)}
                >
                  {emoji}
                </button>
              ))}
            </span>
          </div>
        )}
      </div>
      {isOwn && (
        <span className="chat-avatar chat-avatar--own" aria-hidden style={{ background: avatarColor }}>
          {initialsFromName(displayName)}
        </span>
      )}
    </article>
  );
}
