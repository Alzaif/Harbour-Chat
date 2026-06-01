# Harbour Chat Phase 4 - WebRTC + SFU MVP

Phase 4 upgrades voice channels from participation state to real media transport.

## Scope

Included:

- WebRTC audio capture (`getUserMedia`) and client transport bootstrapping
- Voice signaling request/response envelopes with correlation IDs
- SFU control-plane adapter abstraction (`VoiceMediaPort`) backed by mediasoup service calls
- TURN/STUN server configuration for local development
- Voice media session persistence metadata

Out of scope in this phase:

- Video tracks and screen sharing
- Cross-region SFU scaling
- Production-grade mediasoup cluster orchestration

## Contracts

New event/contract schemas:

- `harbour.voice-session-created.v1`
- `harbour.voice-transport-created.v1`
- `harbour.voice-producer-created.v1`
- `harbour.voice-consumer-created.v1`
- `harbour.voice-ice-candidate.v1`
- `harbour.voice-error.v1`

New signaling envelope contracts:

- `signal_request` with `request_id`, `kind`, `payload`
- `signal_response` with `request_id`, `kind`, `ok`, `payload|error`

## API Signaling Surface

Voice control-plane endpoints:

- `POST /api/channels/:id/voice/session`
- `POST /api/channels/:id/voice/transports`
- `POST /api/channels/:id/voice/transports/:transport_id/connect`
- `POST /api/channels/:id/voice/producers`
- `POST /api/channels/:id/voice/consumers`
- `POST /api/channels/:id/voice/transports/:transport_id/ice-candidates`
- `POST /api/channels/:id/voice/transports/:transport_id/restart-ice`

All routes preserve existing gateway trust-boundary checks and channel membership authorization.

## Persistence

Migration `005_voice_media_sessions.sql` adds:

- `voice_media_sessions`
- `voice_media_transports`
- `voice_media_producers`
- `voice_media_consumers`

These tables track control-plane session metadata, transport lifecycle, and producer/consumer rows.

## Frontend

Voice media client module:

- `web/src/presentation/voice/useVoiceMediaClient.ts`

Responsibilities:

- capture local microphone stream
- bootstrap voice session + transport
- establish producer with signaling API
- expose connection state (`idle`, `connecting`, `connected`, `reconnecting`, `failed`)

## Local Infra

Compose now includes:

- `harbour-chat-sfu` (Node mediasoup service)
- `harbour-turn` (coturn)

Environment settings are documented in:

- `harbour-infra/compose/.env.example`
- `harbour-chat/config/env.example`

Key Phase 4 local defaults:

- `CHAT_VOICE_SFU_BASE_URL=http://harbour-chat-sfu:4000`
- `CHAT_VOICE_TURN_URLS=stun:turn.harbour.local:13478,turn:turn.harbour.local:13478?transport=udp`
- `TURN_STATIC_AUTH_SECRET`, `TURN_LISTEN_PORT`, `TURN_RELAY_MIN_PORT`, `TURN_RELAY_MAX_PORT`
- `CHAT_VOICE_TURN_SECRET`, `CHAT_VOICE_TURN_TTL_SECONDS`

## Validation

Tests now cover:

- contract stability for Phase 4 voice signaling schemas
- integration flow for session + transport signaling endpoints
- existing authz and failure behavior remains enforced

## Local acceptance runbook

1. Start stack with `harbour-chat`, `harbour-chat-sfu`, and `harbour-turn`.
2. Open three browser sessions as different users.
3. Join the same voice channel from all users and confirm bidirectional audio.
4. Move one user to another voice channel and verify channel audio isolation.
5. Toggle mute/deafen and verify state propagation to the other users.
6. Refresh one session and verify reconnect without requiring server restart.
