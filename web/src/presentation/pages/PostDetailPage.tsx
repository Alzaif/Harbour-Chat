import { useCallback, useEffect, useState, type FormEvent } from 'react';
import { Link, useParams } from 'react-router-dom';
import { api } from '../../api/client';
import type { Post, PostComment, VoteValue } from '../../api/types';
import { SharePickerModal } from '../components/SharePickerModal';
import { VoteControls } from '../components/VoteControls';

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

interface CommentNodeProps {
  comment: PostComment;
  depth: number;
  replyToId: string | null;
  replyBody: string;
  submitting: boolean;
  onReply: (commentId: string) => void;
  onCancelReply: () => void;
  onReplyBodyChange: (value: string) => void;
  onSubmitReply: (parentId: string) => void;
}

function CommentNode({
  comment,
  depth,
  replyToId,
  replyBody,
  submitting,
  onReply,
  onCancelReply,
  onReplyBodyChange,
  onSubmitReply,
}: CommentNodeProps) {
  const indent = Math.min(depth, 8);
  const isReplying = replyToId === comment.id;
  const deleted = Boolean(comment.deletedAt) || comment.body === '[deleted]';

  return (
    <li
      className="post-thread__comment"
      style={{ marginLeft: indent === 0 ? 0 : `${indent * 0.85}rem` }}
    >
      <div className="post-thread__comment-meta">
        <span>{comment.authorDisplayName ?? comment.authorUserId}</span>
        <time dateTime={comment.createdAt}>{formatWhen(comment.createdAt)}</time>
      </div>
      <p className={`post-thread__comment-body${deleted ? ' post-thread__comment-body--deleted' : ''}`}>
        {comment.body}
      </p>
      {!deleted && depth < 8 && (
        <button type="button" className="post-thread__reply-btn" onClick={() => onReply(comment.id)}>
          Reply
        </button>
      )}
      {isReplying && (
        <form
          className="post-thread__composer post-thread__composer--inline"
          onSubmit={(e) => {
            e.preventDefault();
            onSubmitReply(comment.id);
          }}
        >
          <textarea
            value={replyBody}
            onChange={(e) => onReplyBodyChange(e.target.value)}
            rows={2}
            placeholder="Write a reply…"
            aria-label="Reply"
            required
          />
          <div className="post-thread__composer-actions">
            <button type="button" onClick={onCancelReply}>
              Cancel
            </button>
            <button type="submit" disabled={submitting || !replyBody.trim()}>
              {submitting ? 'Posting…' : 'Reply'}
            </button>
          </div>
        </form>
      )}
      {comment.replies.length > 0 && (
        <ul className="post-thread__list">
          {comment.replies.map((child) => (
            <CommentNode
              key={child.id}
              comment={child}
              depth={depth + 1}
              replyToId={replyToId}
              replyBody={replyBody}
              submitting={submitting}
              onReply={onReply}
              onCancelReply={onCancelReply}
              onReplyBodyChange={onReplyBodyChange}
              onSubmitReply={onSubmitReply}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

export function PostDetailPage() {
  const { postId } = useParams<{ postId: string }>();
  const [post, setPost] = useState<Post | null>(null);
  const [comments, setComments] = useState<PostComment[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [topBody, setTopBody] = useState('');
  const [replyToId, setReplyToId] = useState<string | null>(null);
  const [replyBody, setReplyBody] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [voting, setVoting] = useState(false);
  const [shareOpen, setShareOpen] = useState(false);
  const [shareNotice, setShareNotice] = useState<string | null>(null);

  const load = useCallback(async (id: string) => {
    const [p, c] = await Promise.all([api.getBoardPost(id), api.listBoardComments(id)]);
    setPost(p);
    setComments(c);
  }, []);

  useEffect(() => {
    if (!postId) return;
    setLoading(true);
    void load(postId)
      .catch((e) => setError(e instanceof Error ? e.message : 'Failed to load post'))
      .finally(() => setLoading(false));
  }, [load, postId]);

  const onVote = async (value: VoteValue) => {
    if (!post) return;
    const previous = post;
    setPost(optimisticVote(post, value));
    setVoting(true);
    try {
      const next = await api.voteBoardPost(post.id, value);
      setPost(next);
    } catch (e) {
      setPost(previous);
      setError(e instanceof Error ? e.message : 'Vote failed');
    } finally {
      setVoting(false);
    }
  };

  const submitTop = async (e: FormEvent) => {
    e.preventDefault();
    if (!postId || !topBody.trim()) return;
    setSubmitting(true);
    try {
      await api.createBoardComment(postId, { body: topBody.trim() });
      setTopBody('');
      const c = await api.listBoardComments(postId);
      setComments(c);
      setPost((p) => (p ? { ...p, commentCount: p.commentCount + 1 } : p));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Comment failed');
    } finally {
      setSubmitting(false);
    }
  };

  const submitReply = async (parentId: string) => {
    if (!postId || !replyBody.trim()) return;
    setSubmitting(true);
    try {
      await api.createBoardComment(postId, {
        body: replyBody.trim(),
        parentCommentId: parentId,
      });
      setReplyBody('');
      setReplyToId(null);
      const c = await api.listBoardComments(postId);
      setComments(c);
      setPost((p) => (p ? { ...p, commentCount: p.commentCount + 1 } : p));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Reply failed');
    } finally {
      setSubmitting(false);
    }
  };

  if (!postId) {
    return (
      <section className="board-page feed-page">
        <p className="board-page__hint">Missing post.</p>
      </section>
    );
  }

  return (
    <section className="board-page feed-page post-detail" aria-labelledby="post-heading">
      <header className="post-detail__header">
        <Link to="/feed" className="post-detail__back">
          ← Board
        </Link>
        {post && (
          <button type="button" className="post-detail__share" onClick={() => setShareOpen(true)}>
            Share
          </button>
        )}
      </header>

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

      <div className="feed-scroll post-thread">
        {loading || !post ? (
          <p className="board-page__hint">{loading ? 'Loading…' : 'Post not found.'}</p>
        ) : (
          <>
            <article className="post-thread__post">
              <VoteControls
                score={post.score}
                myVote={post.myVote}
                disabled={voting}
                onVote={onVote}
              />
              <div className="post-thread__post-body">
                {post.title && (
                  <h1 id="post-heading" className="post-thread__title">
                    {post.title}
                  </h1>
                )}
                {!post.title && (
                  <h1 id="post-heading" className="visually-hidden">
                    Post
                  </h1>
                )}
                <p className="post-thread__body">{post.body}</p>
                {post.linkUrl && (
                  <a
                    className="post-thread__link"
                    href={post.linkUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {post.previewSiteName ?? post.linkUrl}
                  </a>
                )}
                <footer className="post-thread__byline">
                  <span>{post.authorDisplayName ?? post.authorUserId}</span>
                  <time dateTime={post.createdAt}>{formatWhen(post.createdAt)}</time>
                  <span>
                    {post.commentCount}{' '}
                    {post.commentCount === 1 ? 'comment' : 'comments'}
                  </span>
                </footer>
              </div>
            </article>

            <form className="post-thread__composer" onSubmit={submitTop}>
              <label htmlFor="top-comment">Add a comment</label>
              <textarea
                id="top-comment"
                value={topBody}
                onChange={(e) => setTopBody(e.target.value)}
                rows={3}
                placeholder="What are your thoughts?"
                required
              />
              <div className="post-thread__composer-actions">
                <button type="submit" disabled={submitting || !topBody.trim()}>
                  {submitting && !replyToId ? 'Posting…' : 'Comment'}
                </button>
              </div>
            </form>

            {comments.length === 0 ? (
              <p className="board-page__hint">No comments yet.</p>
            ) : (
              <ul className="post-thread__list">
                {comments.map((c) => (
                  <CommentNode
                    key={c.id}
                    comment={c}
                    depth={0}
                    replyToId={replyToId}
                    replyBody={replyBody}
                    submitting={submitting}
                    onReply={(id) => {
                      setReplyToId(id);
                      setReplyBody('');
                    }}
                    onCancelReply={() => {
                      setReplyToId(null);
                      setReplyBody('');
                    }}
                    onReplyBodyChange={setReplyBody}
                    onSubmitReply={(parentId) => {
                      void submitReply(parentId);
                    }}
                  />
                ))}
              </ul>
            )}
          </>
        )}
      </div>

      {shareOpen && post && (
        <SharePickerModal
          post={post}
          onClose={() => setShareOpen(false)}
          onShared={(label) => {
            setShareNotice(`Shared to ${label}`);
            setShareOpen(false);
            window.setTimeout(() => setShareNotice(null), 3000);
          }}
        />
      )}
    </section>
  );
}
