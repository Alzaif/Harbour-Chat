INSERT OR IGNORE INTO servers (id, name, icon_url, owner_user_id, created_at, updated_at)
VALUES (
    '00000000-0000-4000-8000-000000000001',
    'Harbour Home',
    NULL,
    'system',
    0,
    0
);

INSERT OR IGNORE INTO channels (
    id, server_id, category_id, type, name, position, created_at, updated_at
)
VALUES (
    '00000000-0000-4000-8000-000000000002',
    '00000000-0000-4000-8000-000000000001',
    NULL,
    'text',
    'general',
    0,
    0,
    0
);
