import { useCallback, useEffect, useState } from 'react';
import { api } from '../../api/client';
import type { FeedPeriod, FeedSection, Post, VoteValue } from '../../api/types';
import { useChatWebSocket } from '../../hooks/useChatWebSocket';
import { SharePickerModal } from '../components/SharePickerModal';
import { StoryRow } from '../components/StoryRow';

const BOARD_TOPIC = '__board__';

const PERIODS: { id: FeedPeriod; label: string }[] = [
  { id: 'hour', label: '1 hour' },
  { id: 'day', label: '24 hours' },
  { id: 'week', label: 'Week' },
  { id: 'month', label: 'Month' },
  { id: 'year', label: 'Year' },
  { id: 'all', label: 'All time' },
];

function mastheadDate(): string {
  return new Date().toLocaleDateString(undefined, {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}

function patchPostInSections(sections: FeedSection[], next: Post): FeedSection[] {
  return sections.map((section) => ({
    ...section,
    posts: section.posts.map((p) => (p.id === next.id ? next : p)),
  }));
}

function optimisticVote(post: Post, value: VoteValue): Post {
  const prev = post.myVote;
  const nextValue: VoteValue = value !== 0 && prev === value ? 0 : value;
  let upvotes = post.upvotes;
  let downvotes = post.downvotes;
  if (prev === 1) upvotes -= 1;
  if (prev === -1) downvotes -= 1;
  if (nextValue === 1) upvotes += 1;
  if (nextValue === -1) downvotes += 1;
  return {
    ...post,
    myVote: nextValue,
    upvotes,
    downvotes,
    score: upvotes - downvotes,
  };
}

export function BoardFeedPage() {
  const [period, setPeriod] = useState<FeedPeriod>('day');
  const [sections, setSections] = useState<FeedSection[]>([]);
  const [sharePost, setSharePost] = useState<Post | null>(null);
  const [shareNotice, setShareNotice] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCompose, setShowCompose] = useState(false);
  const [title, setTitle] = useState('');
  const [body, setBody] = useState('');
  const [linkUrl, setLinkUrl] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [votingId, setVotingId] = useState<string | null>(null);

  const loadFeed = useCallback(async (p: FeedPeriod) => {
    const feed = await api.listBoardPosts(p);
    setSections(Array.isArray(feed.sections) ? feed.sections : []);
  }, []);

  useEffect(() => {
    setLoading(true);
    void loadFeed(period)
      .catch((e) => setError(e instanceof Error ? e.message : 'Failed to load feed'))
      .finally(() => setLoading(false));
  }, [loadFeed, period]);

  useChatWebSocket([BOARD_TOPIC], (ev) => {
    if (ev.type === 'post_created') {
      setSections((prev) => {
        if (prev.some((s) => s.posts.some((p) => p.id === ev.post.id))) return prev;
        if (prev.length === 0) {
          return [{ kind: 'top', posts: [ev.post] }];
        }
        const [first, ...rest] = prev;
        return [{ ...first, posts: [ev.post, ...first.posts] }, ...rest];
      });
    }
  });

  const submitPost = async () => {
    const trimmed = body.trim();
    if (!trimmed) return;
    setSubmitting(true);
    try {
      const post = await api.createBoardPost({
        title: title.trim() || undefined,
        body: trimmed,
        link_url: linkUrl.trim() || undefined,
      });
      setSections((prev) => {
        if (prev.length === 0) return [{ kind: 'top', posts: [post] }];
        const [first, ...rest] = prev;
        return [
          {
            ...first,
            posts: [post, ...first.posts.filter((p) => p.id !== post.id)],
          },
          ...rest,
        ];
      });
      setShowCompose(false);
      setTitle('');
      setBody('');
      setLinkUrl('');
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Post failed');
    } finally {
      setSubmitting(false);
    }
  };

  const onVote = async (postId: string, value: VoteValue) => {
    const current = sections.flatMap((s) => s.posts).find((p) => p.id === postId);
    if (!current) return;
    const optimistic = optimisticVote(current, value);
    setSections((prev) => patchPostInSections(prev, optimistic));
    setVotingId(postId);
    try {
      const next = await api.voteBoardPost(postId, value);
      setSections((prev) => patchPostInSections(prev, next));
    } catch (e) {
      setSections((prev) => patchPostInSections(prev, current));
      setError(e instanceof Error ? e.message : 'Vote failed');
    } finally {
      setVotingId(null);
    }
  };

  const hasPosts = sections.some((s) => s.posts.length > 0);

  return (
    <section className="board-page feed-page news-feed" aria-labelledby="feed-heading">
      <header className="news-masthead">
        <div className="news-masthead__brand">
          <h1 id="feed-heading">Board</h1>
          <p className="news-masthead__date">{mastheadDate()}</p>
        </div>
        <button type="button" className="board-post-btn" onClick={() => setShowCompose(true)}>
          Post
        </button>
      </header>

      <nav className="news-periods" aria-label="Feed period">
        {PERIODS.map((p) => (
          <button
            key={p.id}
            type="button"
            className={`news-periods__tab${period === p.id ? ' news-periods__tab--active' : ''}`}
            aria-pressed={period === p.id}
            onClick={() => setPeriod(p.id)}
          >
            {p.label}
          </button>
        ))}
      </nav>

      {shareNotice && (
        <p className="feed-share-notice" role="status">
          {shareNotice}
        </p>
      )}

      {error && (
        <div className="chat-error" role="alert">
          {error}
          <button type="button" onClick={() => setError(null)}>
            Dismiss
          </button>
        </div>
      )}

      <div className="feed-scroll">
        {loading ? (
          <p className="board-page__hint">Loading…</p>
        ) : !hasPosts ? (
          <p className="board-page__hint">No posts yet. Tap Post to share something.</p>
        ) : (
          sections.map((section) => (
            <section key={section.kind + (section.label ?? '')} className="news-section">
              {section.kind === 'older' && section.label && (
                <div className="news-divider" role="separator">
                  <span>{section.label}</span>
                </div>
              )}
              <ul className="news-list">
                {section.posts.map((post) => (
                  <li key={post.id}>
                    <StoryRow
                      post={post}
                      onVote={onVote}
                      onShare={setSharePost}
                      voting={votingId === post.id}
                    />
                  </li>
                ))}
              </ul>
            </section>
          ))
        )}
      </div>

      {showCompose && (
        <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
          <form
            className="chat-modal feed-compose board-sheet"
            onSubmit={(e) => {
              e.preventDefault();
              void submitPost();
            }}
          >
            <h2>New post</h2>
            <input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Title (optional)"
              aria-label="Title"
            />
            <textarea
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder="What do you want to share?"
              rows={4}
              required
              aria-label="Body"
            />
            <input
              value={linkUrl}
              onChange={(e) => setLinkUrl(e.target.value)}
              placeholder="Link URL (optional)"
              type="url"
              aria-label="Link URL"
            />
            <div className="chat-modal__actions">
              <button type="button" onClick={() => setShowCompose(false)}>
                Cancel
              </button>
              <button type="submit" disabled={submitting || !body.trim()}>
                {submitting ? 'Posting…' : 'Post'}
              </button>
            </div>
          </form>
        </div>
      )}

      {sharePost && (
        <SharePickerModal
          post={sharePost}
          onClose={() => setSharePost(null)}
          onShared={(label) => {
            setShareNotice(`Shared to ${label}`);
            setSharePost(null);
            window.setTimeout(() => setShareNotice(null), 3000);
          }}
        />
      )}
    </section>
  );
}
