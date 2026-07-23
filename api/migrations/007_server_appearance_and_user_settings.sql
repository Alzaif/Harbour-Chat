ALTER TABLE servers ADD COLUMN description TEXT;
ALTER TABLE servers ADD COLUMN card_color TEXT;

CREATE TABLE user_preferences (
    user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    push_to_talk INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL
);
