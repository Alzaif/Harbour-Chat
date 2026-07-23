import { useNavigate } from 'react-router-dom';
import type { Post, VoteValue } from '../../api/types';
import { VoteControls } from './VoteControls';

export interface StoryRowProps {
  post: Post;
  onVote: (postId: string, value: VoteValue) => void;
  onShare?: (post: Post) => void;
  voting?: boolean;
}

function headline(post: Post): string {
  if (post.title?.trim()) return post.title.trim();
  const body = post.body.trim();
  if (body.length <= 120) return body;
  return `${body.slice(0, 119)}…`;
}

function formatWhen(iso: string): string {
  try {
    return new Date(iso).toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return iso;
  }
}

export function StoryRow({ post, onVote, onShare, voting }: StoryRowProps) {
  const navigate = useNavigate();
  const kicker = post.previewSiteName ?? (post.linkUrl ? 'Link' : null);

  return (
    <article className="news-story">
      <VoteControls
        score={post.score}
        myVote={post.myVote}
        disabled={voting}
        onVote={(value) => onVote(post.id, value)}
      />
      <div className="news-story__main">
        {kicker && (
          <p className="news-story__kicker">
            {post.linkUrl ? (
              <a href={post.linkUrl} target="_blank" rel="noopener noreferrer">
                {kicker}
              </a>
            ) : (
              kicker
            )}
          </p>
        )}
        <button
          type="button"
          className="news-story__headline"
          onClick={() => navigate(`/feed/${post.id}`)}
        >
          {headline(post)}
        </button>
        <footer className="news-story__byline">
          <span>{post.authorDisplayName ?? post.authorUserId}</span>
          <time dateTime={post.createdAt}>{formatWhen(post.createdAt)}</time>
          <span>
            {post.commentCount} {post.commentCount === 1 ? 'comment' : 'comments'}
          </span>
          {onShare && (
            <button
              type="button"
              className="news-story__share"
              onClick={() => onShare(post)}
            >
              Share
            </button>
          )}
        </footer>
      </div>
    </article>
  );
}
