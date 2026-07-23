# Board feed ranking and comment tree

## Ranking (Top within period)

Period windows: `hour` (1h), `day` (24h), `week` (7d), `month` (30d), `year` (365d), `all`.

`GET /api/board/posts?period=` returns a **sectioned** payload:

1. **`top`** — posts with `created_at` in the window, ordered by `score DESC`, then `created_at DESC`.
2. **`older`** — posts older than the window, newest-first, with label `Posts older than …` (omitted for `all`).

`score` is denormalized on `posts` as `upvotes - downvotes`, maintained transactionally with `post_votes`.

## Votes

`POST /api/board/posts/:id/vote` with `{ value: 1 | -1 | 0 }`:

- Same value as current vote → clear (equivalent to `0`).
- Opposite value → flip.
- Explicit `0` → clear.

Response is the updated post including `myVote`.

## Comments

`post_comments` supports nested replies via `parent_comment_id`. Soft-deleted rows keep tree shape; the API substitutes body `[deleted]`.

List endpoint returns a nested tree (flat fetch + application nesting). Soft depth cap for new replies: 8.

Realtime score/comment fan-out is out of scope for MVP; UI refreshes on navigate and uses optimistic votes.
