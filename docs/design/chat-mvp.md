# Harbour Chat MVP (Phase 1)

**Runtime:** Rust API (`api/`) + React SPA (`web/`). Product design: [discord-app.md](../../../discord-app.md). **Phase 2:** [chat-phase-2-mvp.md](chat-phase-2-mvp.md).

## Auth

- Scope: `app:chat`
- Host: `chat.harbour.local`
- ForwardAuth + `X-Harbour-*` headers (see `api/src/contracts/gateway_headers.rs`)

## Seeded data

| Id | Entity |
|----|--------|
| `00000000-0000-4000-8000-000000000001` | Harbour Home server |
| `00000000-0000-4000-8000-000000000002` | `#general` text channel |

First access to Harbour Home auto-adds the current user as `member`.

## API (summary)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/servers` | List servers for user |
| GET | `/api/servers/:id` | Server + channels |
| POST | `/api/servers/:id/members` | Add member (`userId`) |
| GET | `/api/servers/:id/members` | List members |
| GET | `/api/channels/:id/messages` | Message history (nested `reply_to` when set) |
| POST | `/api/channels/:id/messages` | Send message (`content`, optional `reply_to_message_id`) |
| PATCH | `/api/messages/:id` | Edit |
| DELETE | `/api/messages/:id` | Soft delete |
| POST | `/api/messages/:id/reactions` | Toggle reaction |
| POST | `/api/channels/:id/read` | Mark read |
| POST | `/api/dms/:userId` | Open DM |
| GET | `/api/ws` | WebSocket subscribe |
| GET | `/health` | Service health probe |
| GET | `/version` | Build/package version |

Board share/forward picker: `GET /api/board/share-targets` (see [board-app.md](../../../board-app.md)).

### Message JSON (replies)

- Migration `010_message_reply.sql` adds nullable `messages.reply_to_message_id` (FK → `messages.id`, `ON DELETE SET NULL`).
- Send/list responses may include `reply_to_message_id` and nested `reply_to` (`ReplyPreview`: `id`, `author_user_id`, `author_display_name`, `content`, `deleted_at`).
- UI: right-click Copy / Reply / Forward; composer reply bar; Forward posts a new message via share-targets (not a reply link).

## WebSocket protocol (MVP)

- **Client subscribe frame:** `{"type":"subscribe","channelIds":["<channel-id>"]}`
- **Server event framing:** realtime payloads are serialized from `RealtimeEvent` in `api/src/domain/ports/realtime_publisher.rs` (`serde(tag = "type", rename_all = "snake_case")`).
- **Event `type` values:** `message_created`, `message_updated`, `message_deleted`, `reaction_updated`.

## Dev ports

| Service | Port |
|---------|------|
| API | 3004 |
| Vite | 5177 |

## Integration events

- `harbour.message-sent.v1` — contract in `api/src/contracts/events.rs`
  - `message_id`
  - `channel_id`
  - `server_id`
  - `author_user_id`
  - `occurred_at`
