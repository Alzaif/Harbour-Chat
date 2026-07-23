-- Nested Reddit-style comments on Board posts.
CREATE TABLE post_comments (
    id TEXT PRIMARY KEY NOT NULL,
    post_id TEXT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    author_user_id TEXT NOT NULL REFERENCES users(id),
    parent_comment_id TEXT REFERENCES post_comments(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    edited_at INTEGER,
    deleted_at INTEGER
);

CREATE INDEX post_comments_post_created_idx ON post_comments(post_id, created_at);
CREATE INDEX post_comments_parent_idx ON post_comments(parent_comment_id);
