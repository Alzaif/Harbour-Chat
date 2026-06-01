# Harbour Chat Phase 3 - Media and Presence

Phase 3 introduces lightweight real-time social context and synchronous voice coordination:

- Typing indicators for channels and DMs
- Presence states (`online`, `idle`, `dnd`, `offline`) at server scope
- Voice channel participation (join/leave, mute/deafen, roster updates)

## Scope

Included:

1. Presence API and realtime events
2. Typing API with short TTL and realtime updates
3. Voice channel participant state API and realtime updates
4. UI surfaces for typing hints, presence dots, and voice controls

Not included:

- WebRTC media transport (audio/video streaming)
- Screen sharing
- Voice moderation tooling

## Backend design

### Persistence

Added SQLite tables:

- `presence_states(server_id, user_id, status, updated_at)`
- `typing_states(channel_id, user_id, expires_at)`
- `voice_participants(channel_id, user_id, muted, deafened, updated_at)`

Typing rows are ephemeral and pruned when fetched if expired.

### HTTP API

- `GET/POST /api/servers/:id/presence`
- `GET/POST /api/channels/:id/typing`
- `GET /api/channels/:id/voice`
- `POST /api/channels/:id/voice/join`
- `POST /api/channels/:id/voice/leave`
- `POST /api/channels/:id/voice/state`

### Realtime events

The websocket stream now emits:

- `typing_started`
- `typing_stopped`
- `presence_changed`
- `voice_participant_updated`

These events are published after successful state writes.

## Frontend design

- Typing indicators render below the timeline and auto-expire.
- Presence status displays as colored dots in the members panel.
- Voice channels appear in a dedicated channel section.
- Voice header controls support join/leave/mute/deafen.
- Voice roster is shown under the message area for active voice channels.

## Security and boundary notes

- Existing Phase 2.5 proxy trust and header validation remain unchanged.
- Voice participation changes are written to audit events (`voice.participant.updated`).
- Membership checks continue to enforce server and channel access before writes.

## Tests

- Contract tests for new events under `api/tests/contracts_events_test.rs`
- Integration tests for presence, typing, and voice endpoints in `api/tests/integration_test.rs`
- Existing web tests updated with API mocks for the new methods
