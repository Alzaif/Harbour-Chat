# Harbour Chat Phase 2 MVP

**Runtime:** Rust API (`api/`) + React SPA (`web/`). Phase 1: [chat-mvp.md](chat-mvp.md). Product design: [discord-app.md](../../../discord-app.md).

## Goals

Phase 2 makes Harbour Chat feel like a daily messenger: **own messages on the right**, admin flows for servers/channels, members panel, reactions, attachments, and unread badges. Security hardening is tracked in **[chat-phase-2-5-security-hardening.md](chat-phase-2-5-security-hardening.md)** and media/presence expansion in **[chat-phase-3-media-presence.md](chat-phase-3-media-presence.md)**.

## Phase 2 scope

| # | Feature | Backend | UI |
|---|---------|---------|-----|
| 1 | Messenger bubbles (own right) | `GET /api/me` | `MessageBubble`, `--own` styles |
| 2 | Create server / channel | Re-enable `POST /api/servers`, `POST .../channels` | Modals + “+” controls |
| 3 | Members drawer | Members include `display_name` | Fourth column |
| 4 | Reactions | `GET` reactions on messages; existing toggle | Picker + counts; WS merge |
| 5 | Attachments | `AttachmentStorePort`, upload/download | Composer attach, image in bubble |
| 6 | Unread badges | Unread counts on server detail | Channel list badges |
| 7 | Timeline polish | (existing pagination) | Infinite scroll, WS edit/delete, timestamps, DMs section |

## API (Phase 2 additions)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/me` | Current user `{ id, email, displayName }` |
| POST | `/api/servers` | Create server |
| POST | `/api/servers/:id/channels` | Create text channel |
| GET | `/api/attachments/:id` | Download attachment (member check) |
| POST | `/api/messages/:id/attachments` | Multipart upload |

Server detail (`GET /api/servers/:id`) includes `unreadByChannelId: Record<channelId, number>`.

Messages include `reactions: { emoji, count, userIds }[]` when listed or sent. Quoted replies (`reply_to_message_id` / `reply_to`) are documented in [chat-mvp.md](chat-mvp.md).

## Bubble layout

- **Others:** left — avatar, name, neutral bubble.
- **You:** right — name above bubble (optional), accent bubble, `author_user_id === me.id`.

## Later phases (out of scope for Phase 2)

- Threads
- Message search (FTS)
- Web push
- Link previews
- Full media stack (WebRTC audio/video transport)

## Delivery PRs

| PR | Focus |
|----|--------|
| P2-1 | `/api/me` + bubbles |
| P2-2 | Create server/channel |
| P2-3 | Members drawer |
| P2-4 | Reactions UI |
| P2-5 | Attachments |
| P2-6 | Unread badges |
| P2-7 | Scroll + WS polish + DMs |
