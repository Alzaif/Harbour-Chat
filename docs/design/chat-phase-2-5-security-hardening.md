# Harbour Chat Phase 2.5 Security Hardening

Status: In progress (maximum-assurance track)

Related: [chat-phase-2-mvp.md](chat-phase-2-mvp.md), [../../../discord-app.md](../../../discord-app.md), [../../../ROADMAP.md](../../../ROADMAP.md)

## Objective

Phase 2.5 upgrades Harbour Chat from feature-complete MVP to a high-assurance security posture with strict transport trust boundaries, encrypted persistence, hardened upload controls, immutable auditability, and explicit verification gates.

## Threat Model Summary

Primary risks addressed in this phase:

- forged identity headers when API is reachable outside trusted edge
- plaintext data recovery from storage volumes/backups
- malicious attachment upload or content-type spoofing
- weak observability for authz and data-access events
- insecure transport downgrade or missing TLS controls

## Controls Implemented in Phase 2.5

### 1) Transport + Trust Boundary

- enforce forwarded HTTPS check in gateway middleware when trusting gateway headers
- optional proxy-shared token (`X-Harbour-Proxy-Token`) required by API before accepting identity headers
- **fail closed at startup** when `TRUST_GATEWAY_HEADERS=true`: `CHAT_TRUSTED_PROXY_TOKEN` and `CHAT_MASTER_KEY_B64` must both be set (`Config::validate_runtime`)
- strict response hardening headers on API responses (`nosniff`, `frame deny`, HSTS, referrer policy)
- attachment download hardened with explicit `Content-Disposition: attachment`
- Traefik dashboard insecure mode disabled in infra config
- Traefik middleware for security headers and proxy-token forwarding added in compose
- chat service env now supports hardened trust settings:
  - `CHAT_REQUIRE_HTTPS_FORWARDED_PROTO`
  - `CHAT_TRUSTED_PROXY_TOKEN`

### 1b) Realtime subscribe authorization

- WebSocket `subscribe` frames are filtered through `ChatService::authorize_realtime_subscribe` before joining broadcast topics
- channel/DM topics require the same membership checks as HTTP read paths (`require_channel_access`)
- board feed topic (`__board__`) is allowed for any authenticated Board user (identity already gated at WS handshake)

### 2) Encryption at Rest (messages + attachments)

- envelope encryption module added: `api/src/infrastructure/security/envelope_crypto.rs`
- message content encrypted before persistence and transparently decrypted on read path
- attachment bytes encrypted before file write and decrypted on read path
- key metadata support through env:
  - `CHAT_MASTER_KEY_B64` (base64 32-byte KEK)
  - `CHAT_MASTER_KEY_ID`
- migration binary added:
  - `cargo run --bin security_migrate` to re-encrypt existing plaintext messages/attachments

### 3) Upload Hardening + Abuse Resistance

- upload MIME allowlist retained and strengthened with magic-byte detection (`infer`)
- rejects declared MIME that does not match detected content
- suspicious malware-marker payload quarantine support (`CHAT_QUARANTINE_SUSPICIOUS_ATTACHMENTS` behavior path)
- filename normalization and truncation to reduce path/encoding abuse

### 4) Immutable Audit Logging

- migration adds `audit_events` table with indexed timestamp and user lookups
- structured security events written for key actions:
  - message send/edit/delete
  - attachment upload/download
- logger also emits structured tracing event (`security_audit` target)

### 5) Identity Boundary (Portcullis)

- ForwardAuth path enforces `X-Forwarded-Proto=https` when configured
- config field added: `REQUIRE_HTTPS_FORWARDED_PROTO`

## Security Control Matrix

| Control | Repo | Enforcement | Test coverage |
|---|---|---|---|
| Trusted proxy boundary | `harbour-chat/api`, `harbour-infra` | forwarded proto + proxy token + edge middleware + startup validation | integration + config unit tests |
| Realtime subscribe ACL | `harbour-chat/api` | WS subscribe filtered by channel membership | integration tests |
| TLS strictness and headers | `harbour-infra`, `harbour-chat/api` | HSTS, nosniff, frame deny, HTTPS-only assumptions | integration tests |
| Message at-rest encryption | `harbour-chat/api` | envelope encryption in service path | existing integration + migration utility |
| Attachment at-rest encryption | `harbour-chat/api` | encrypted blob writes in attachment store | integration tests |
| Content-type spoof defense | `harbour-chat/api` | magic-byte vs declared MIME mismatch rejection | integration tests |
| Security audit trail | `harbour-chat/api` | append-only audit event inserts | migration + runtime logging |
| ForwardAuth HTTPS binding | `portcullis` | reject forward-auth request when proto not https | unit/integration via app flow |

## Rollout Gates

### Gate A: Transport hardening

- `HARBOUR_PROXY_TOKEN` is set in infra environment
- chat receives `CHAT_TRUSTED_PROXY_TOKEN` with same value
- chat receives non-empty `CHAT_MASTER_KEY_B64` (API refuses to start in gateway mode otherwise)
- direct API access path from untrusted network blocked
- smoke test: non-https forwarded proto rejected

### Gate B: Crypto enablement

- `CHAT_MASTER_KEY_B64` provisioned from secret manager/runtime secret
- run `security_migrate` and verify no plaintext remains in message rows
- validate attachment read/write behavior post migration

### Gate C: Upload defense

- magic-byte mismatch rejected
- quarantine path writable and monitored
- size limits enforced in env and tested

### Gate D: Audit and detection

- `audit_events` table populated for sensitive operations
- log forwarding from `security_audit` target attached to platform observability
- alert thresholds defined for auth failures and suspicious upload patterns

## Required Environment Variables

### harbour-chat API

- `CHAT_REQUIRE_HTTPS_FORWARDED_PROTO=true`
- `CHAT_TRUSTED_PROXY_TOKEN=<shared-secret>`
- `CHAT_MASTER_KEY_B64=<base64-32-byte-key>`
- `CHAT_MASTER_KEY_ID=chat-kek-v1`
- `CHAT_ENABLE_SECURITY_AUDIT_LOG=true`
- `CHAT_MAX_ATTACHMENT_BYTES=...`

### portcullis

- `REQUIRE_HTTPS_FORWARDED_PROTO=true`

### harbour-infra

- `HARBOUR_PROXY_TOKEN=<shared-secret>`

## Operational Runbooks

### Key rotation

1. Provision new KEK and set next `CHAT_MASTER_KEY_ID`
2. deploy API with both old/new key support (follow-up improvement)
3. run migration utility to re-encrypt historical data
4. remove old key after verification window

### Incident triage

- inspect `audit_events` by `created_at DESC` and `user_id`
- correlate with `security_audit` logs
- isolate suspicious attachment storage keys from metadata and quarantine

## Follow-up hardening backlog (Phase 2.5b)

- move KEK to Vault/KMS-backed retrieval with runtime refresh
- mTLS between edge and app containers
- add replay protection/idempotency token for selected mutating routes
- integrate real antivirus scanning backend (ClamAV sidecar or external scanner)
- SIEM integration and rule pack automation
