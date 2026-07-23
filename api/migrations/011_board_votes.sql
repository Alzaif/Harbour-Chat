-- Board post votes + denormalized score columns for Top ranking.
ALTER TABLE posts ADD COLUMN upvotes INTEGER NOT NULL DEFAULT 0;
ALTER TABLE posts ADD COLUMN downvotes INTEGER NOT NULL DEFAULT 0;
ALTER TABLE posts ADD COLUMN score INTEGER NOT NULL DEFAULT 0;

CREATE TABLE post_votes (
    post_id TEXT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    value INTEGER NOT NULL CHECK (value IN (-1, 1)),
    PRIMARY KEY (post_id, user_id)
);

CREATE INDEX post_votes_user_idx ON post_votes(user_id);
CREATE INDEX posts_score_created_idx ON posts(score DESC, created_at DESC);
