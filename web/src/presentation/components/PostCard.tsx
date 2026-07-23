import type { Post } from '../../api/types';

export interface PostCardProps {
  post: Post;
  expanded?: boolean;
  onToggleExpand: () => void;
  onShare?: () => void;
}

function previewLabel(post: Post): string {
  return post.previewSiteName ?? post.previewTitle ?? 'Link';
}

export function PostCard({ post, expanded, onToggleExpand, onShare }: PostCardProps) {
  const hasLink = Boolean(post.linkUrl);

  return (
    <article className={`post-card${expanded ? ' post-card--expanded' : ''}`}>
      <button type="button" className="post-card__body" onClick={onToggleExpand}>
        {post.title && <h2 className="post-card__title">{post.title}</h2>}
        <p className="post-card__excerpt">{expanded ? post.body : truncate(post.body, 160)}</p>
        <footer className="post-card__meta">
          <span>{post.authorDisplayName ?? post.authorUserId}</span>
          <time dateTime={post.createdAt}>{formatWhen(post.createdAt)}</time>
        </footer>
      </button>
      {hasLink && (
        <a
          className="post-card__preview"
          href={post.linkUrl!}
          target="_blank"
          rel="noopener noreferrer"
          onClick={(e) => e.stopPropagation()}
        >
          {post.previewImageUrl ? (
            <img src={post.previewImageUrl} alt="" className="post-card__preview-img" />
          ) : (
            <span className="post-card__preview-fallback">{previewLabel(post)}</span>
          )}
        </a>
      )}
      <div className="post-card__actions">
        <button
          type="button"
          className="post-card__share"
          onClick={(e) => {
            e.stopPropagation();
            onShare?.();
          }}
        >
          Share
        </button>
      </div>
    </article>
  );
}

function truncate(text: string, max: number): string {
  if (text.length <= max) return text;
  return `${text.slice(0, max - 1)}…`;
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
