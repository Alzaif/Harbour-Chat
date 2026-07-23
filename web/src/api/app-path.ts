/** Prefix a relative API path with the Vite base (e.g. `/board/`). */
export function apiUrl(path: string): string {
  const base = import.meta.env.BASE_URL;
  const normalized = path.startsWith('/') ? path.slice(1) : path;
  return `${base}${normalized}`;
}

/**
 * URL for a user's avatar image. `version` (e.g. `avatarUpdatedAt`) busts the
 * browser cache after the current user uploads a new photo.
 */
export function userAvatarUrl(userId: string, version?: number | null): string {
  const url = apiUrl(`/api/users/${encodeURIComponent(userId)}/avatar`);
  return version ? `${url}?v=${version}` : url;
}
