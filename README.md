# Harbour Chat

Private group chat for the Harbour platform — **Rust API** (Axum, SQLite, WebSocket) and **React** SPA.

- Host: `harbour.local/board` (path prefix on shell host)
- Scope: `app:board`
- URL: `https://harbour.<zone>:8443/board/`
- Surfaces: Direct, Servers (Party voice), and Board — a newspaper-style household feed with period Top ranking, votes, and nested comments
- Design: [../board-app.md](../board-app.md) (message context menu, reply/forward, Direct dock width, Board feed), [Board feed ranking](docs/design/board-feed-ranking.md) (period Top + older sections, votes, comment tree), [Phase 1 API](docs/design/chat-mvp.md) (`reply_to_message_id` / `ReplyPreview`), [Phase 2 MVP](docs/design/chat-phase-2-mvp.md), [Phase 2.5 Security Hardening](docs/design/chat-phase-2-5-security-hardening.md), [Phase 3 Media and Presence](docs/design/chat-phase-3-media-presence.md), [Phase 4 WebRTC + SFU](docs/design/chat-phase-4-webrtc-sfu.md)

## Prerequisites

- Rust 1.80+ (`rustup`, `cargo`)
- Node.js 22+

## Local development

```bash
cd harbour-chat
cp config/env.example .env   # optional — export vars manually

# Terminal A — API on :3004
cd api && cargo run

# Terminal B — UI on :5177 (proxies /api to :3004)
cd web && npm install && npm run dev
```

Or from repo root: `npm install && npm run dev` (requires `cargo` on PATH).

Grant access:

```bash
./scripts/user-admin.sh users grant-app --email you@example.com --app board
```

## Local auth and gateway trust

`TRUST_GATEWAY_HEADERS` controls identity resolution for both HTTP routes and WebSocket upgrades:

- `TRUST_GATEWAY_HEADERS=false` (default local dev): uses `DEV_USER_ID`, `DEV_USER_EMAIL`, and `DEV_USER_DISPLAY_NAME`.
- `TRUST_GATEWAY_HEADERS=true` (gateway mode): requires forwarded `X-Harbour-*` headers and enforces the `app:board` scope.

Security hardening env vars (Phase 2.5):

- `CHAT_REQUIRE_HTTPS_FORWARDED_PROTO=true`
- `CHAT_TRUSTED_PROXY_TOKEN=<shared-edge-secret>`
- `CHAT_MASTER_KEY_B64=<base64-32-byte-key>`
- `CHAT_MASTER_KEY_ID=chat-kek-v1`
- `CHAT_ENABLE_SECURITY_AUDIT_LOG=true`

To migrate pre-existing plaintext records:

```bash
cd api
cargo run --bin security_migrate
```

## Health and version endpoints

- `GET /health` returns service readiness for platform probes.
- `GET /version` returns package name and version metadata.

## Phase 3 social realtime

Phase 3 adds lightweight realtime social signals without WebRTC media transport:

- Typing indicators: `GET/POST /api/channels/:id/typing`
- Presence states: `GET/POST /api/servers/:id/presence`
- Voice channel state + roster:
  - `GET /api/channels/:id/voice`
  - `POST /api/channels/:id/voice/join`
  - `POST /api/channels/:id/voice/leave`
  - `POST /api/channels/:id/voice/state`

WebSocket events for clients:

- `typing_started`, `typing_stopped`
- `presence_changed`
- `voice_participant_updated`

## Phase 4 voice media transport

Phase 4 introduces WebRTC signaling and media-session control-plane APIs for SFU-backed voice:

- `POST /api/channels/:id/voice/session`
- `POST /api/channels/:id/voice/transports`
- `POST /api/channels/:id/voice/transports/:transport_id/connect`
- `POST /api/channels/:id/voice/producers`
- `POST /api/channels/:id/voice/consumers`
- `POST /api/channels/:id/voice/transports/:transport_id/ice-candidates`
- `POST /api/channels/:id/voice/transports/:transport_id/restart-ice`

Environment vars:

- `CHAT_VOICE_SFU_BASE_URL`
- `CHAT_VOICE_TURN_URLS`
- `CHAT_VOICE_TURN_SECRET`
- `CHAT_VOICE_TURN_TTL_SECONDS`
- `TURN_STATIC_AUTH_SECRET` (shared TURN auth secret used by coturn)
- `TURN_LISTEN_PORT` (default host port `13478` -> container `3478`)
- `TURN_RELAY_MIN_PORT` / `TURN_RELAY_MAX_PORT` (UDP relay range)
- `CHAT_SFU_HTTP_PORT` (host port for SFU control endpoint)
- `CHAT_SFU_RTP_UDP_START` / `CHAT_SFU_RTP_UDP_END` (SFU RTP UDP range)

Local infra defaults for Phase 4 live in `harbour-infra/compose/.env.example` and `harbour-infra/compose/docker-compose.yml`. `harbour-chat-sfu` runs the mediasoup-based SFU service for local development.

Proxy notes for WebSocket signaling:

- Traefik routes board API traffic on a **dedicated `board-api` router** (priority 150) straight to the Rust API on **port 3000**, after `board-stripprefix` and ForwardAuth. Static SPA assets use the `board` router (priority 80) to nginx on port 80. This prevents `/board/api/*` from ever hitting nginx's SPA `try_files` fallback.
- Traefik routes `Path('/board/api/ws')` on `board-ws` (priority 200), also to port 3000, **without** ForwardAuth (HTTP/1.1 upgrade requests break ForwardAuth).
- Container nginx still accepts both `/api/*` and `/board/api/*` for local/dev paths where Traefik is not in front.
- The chat API validates the `harbour_session` cookie by calling Portcullis `/auth/forward` internally (`CHAT_PORTCULLIS_FORWARD_URL`).
- Nginx has dedicated `location` blocks for `/api/ws` and `/board/api/ws` with WebSocket upgrade headers and long-lived proxy timeouts.
- The in-memory realtime hub now explicitly responds to websocket `Ping` frames with `Pong`, ignores `Pong` frames, and exits cleanly on `Close` frames.
- If clients still disconnect during idle periods, verify upstream proxy/LB websocket timeout and control-frame handling (`Ping`/`Pong`) are not stripping or buffering frames.

## Tests

```bash
cd api && cargo test
cd web && npm test
```

## Layout

| Path | Role |
|------|------|
| `api/` | Rust hexagonal backend |
| `web/` | React + Vite presentation |
| `docker/` | nginx + entrypoint |
