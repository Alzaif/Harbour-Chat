CREATE TABLE IF NOT EXISTS voice_media_sessions (
    session_id TEXT PRIMARY KEY NOT NULL,
    channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    closed_at INTEGER
);

CREATE INDEX IF NOT EXISTS voice_media_sessions_channel_idx
    ON voice_media_sessions(channel_id, created_at DESC);

CREATE TABLE IF NOT EXISTS voice_media_transports (
    transport_id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES voice_media_sessions(session_id) ON DELETE CASCADE,
    direction TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    connected_at INTEGER
);

CREATE INDEX IF NOT EXISTS voice_media_transports_session_idx
    ON voice_media_transports(session_id);

CREATE TABLE IF NOT EXISTS voice_media_producers (
    producer_id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES voice_media_sessions(session_id) ON DELETE CASCADE,
    transport_id TEXT NOT NULL REFERENCES voice_media_transports(transport_id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    closed_at INTEGER
);

CREATE TABLE IF NOT EXISTS voice_media_consumers (
    consumer_id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES voice_media_sessions(session_id) ON DELETE CASCADE,
    transport_id TEXT NOT NULL REFERENCES voice_media_transports(transport_id) ON DELETE CASCADE,
    producer_id TEXT NOT NULL REFERENCES voice_media_producers(producer_id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    closed_at INTEGER
);
