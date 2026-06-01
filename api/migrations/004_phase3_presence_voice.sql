CREATE TABLE IF NOT EXISTS presence_states (
    server_id TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('online', 'idle', 'dnd', 'offline')),
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (server_id, user_id)
);

CREATE INDEX IF NOT EXISTS presence_states_server_idx
    ON presence_states(server_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS typing_states (
    channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at INTEGER NOT NULL,
    PRIMARY KEY (channel_id, user_id)
);

CREATE INDEX IF NOT EXISTS typing_states_channel_idx
    ON typing_states(channel_id, expires_at DESC);

CREATE TABLE IF NOT EXISTS voice_participants (
    channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    muted INTEGER NOT NULL DEFAULT 0,
    deafened INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (channel_id, user_id)
);

CREATE INDEX IF NOT EXISTS voice_participants_channel_idx
    ON voice_participants(channel_id, updated_at DESC);
