# Harbour Chat — Profile Avatars

Adds user-uploadable profile pictures. Avatars are owned and stored by
`harbour-chat` (the gateway/Portcullis only supplies `id`, `email`,
`displayName`, and scopes — no avatar), mirroring the message-attachment
storage pattern.

## Backend

### Persistence

Migration `009_user_avatars.sql`:

```
user_avatars(user_id PK -> users.id, storage_key, mime_type, size_bytes, updated_at)
```

Bytes are stored **encrypted at rest** on the filesystem at
`{CHAT_DATA_DIR}/avatars/{storage_key}` (same `EnvelopeCrypto` as attachments).
Uploading a new avatar replaces the row and best-effort deletes the previous
file. Size limit reuses `CHAT_MAX_ATTACHMENT_BYTES`.

### Port + adapter

- `domain/ports/avatar_store.rs` — `AvatarStore` port with `save`, `read`,
  `meta`, plus the `AvatarMeta { mime_type, size_bytes, updated_at }` value.
- `infrastructure/storage/local_avatar_store.rs` — `LocalAvatarStore`. Validates
  image-only MIME (`image/jpeg|png|gif|webp`), sniffs content with `infer`, and
  rejects declared/detected mismatches.
- Wired in `infrastructure/state.rs` as `AppState.avatars`.

### HTTP API

- `POST /api/me/avatar` (multipart, field `file`) → `{ mimeType, sizeBytes, avatarUpdatedAt }`.
  Only the authenticated user can set their own avatar.
- `GET /api/users/{id}/avatar` → raw image bytes (`Content-Type` from stored
  mime, `Cache-Control: private, max-age=300`, `X-Content-Type-Options: nosniff`).
  Returns `404` when no avatar is set. Any authenticated user may read any
  user's avatar.
- `GET /api/me` now also returns `avatarUpdatedAt` (ms epoch or null) for
  cache-busting the current user's own image.

## Frontend

- `api.uploadAvatar(file)` and `CurrentUser.avatarUpdatedAt`
  (`AvatarUploadResult` contract in `api/types.ts`).
- `userAvatarUrl(userId, version?)` (`api/app-path.ts`) builds the serve URL and
  appends `?v={version}` for cache-busting.
- `presentation/components/Avatar.tsx` — reusable avatar that renders the
  uploaded image and **falls back to colored initials** when there is no user id
  or the image 404s (`onError`). Used by `MessageBubble`, `MembersPanel`, and
  `UserDock` (the current user passes `version={avatarUpdatedAt}`).
- `SettingsModal` gains a "Profile photo" row (hidden file input + preview) that
  calls `api.uploadAvatar` and notifies its parent via `onProfileUpdated` so the
  dock refreshes immediately.

## Tests

- Integration (`api/tests/integration_test.rs`): `avatar_upload_and_serve_roundtrip`
  (404 before upload → upload → serve bytes match → `me.avatarUpdatedAt` set) and
  `avatar_upload_rejects_non_image`.
- Unit (web): `Avatar.test.tsx` (image vs initials fallback, cache-bust version)
  and `SettingsModal.test.tsx` avatar-upload case.
