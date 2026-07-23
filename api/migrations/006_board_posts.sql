CREATE TABLE posts (
    id TEXT PRIMARY KEY NOT NULL,
    author_user_id TEXT NOT NULL REFERENCES users(id),
    title TEXT,
    body TEXT NOT NULL,
    link_url TEXT,
    preview_title TEXT,
    preview_description TEXT,
    preview_image_url TEXT,
    preview_site_name TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX posts_created_idx ON posts(created_at DESC);
