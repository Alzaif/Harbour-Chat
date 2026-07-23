import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { api } from './client';

const jsonResponse = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });

const htmlResponse = (status = 200) =>
  new Response('<!doctype html><html><body>app shell</body></html>', {
    status,
    headers: { 'content-type': 'text/html' },
  });

describe('api request', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn());
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('parses JSON on a successful response', async () => {
    vi.mocked(fetch).mockResolvedValue(jsonResponse({ id: 'u1', email: 'a@b', displayName: 'A' }));
    await expect(api.me()).resolves.toEqual({ id: 'u1', email: 'a@b', displayName: 'A' });
  });

  it('returns undefined for a 204 response', async () => {
    vi.mocked(fetch).mockResolvedValue(new Response(null, { status: 204 }));
    await expect(api.leaveVoice('c1')).resolves.toBeUndefined();
  });

  it('throws the API error message on a JSON error response', async () => {
    vi.mocked(fetch).mockResolvedValue(jsonResponse({ error: 'not a member' }, 403));
    await expect(api.joinVoice('c1')).rejects.toThrow('not a member');
  });

  it('throws an HTTP status when an error response is not JSON', async () => {
    vi.mocked(fetch).mockResolvedValue(htmlResponse(502));
    await expect(api.joinVoice('c1')).rejects.toThrow('HTTP 502');
  });

  it('does not surface a raw JSON parse error when a 2xx response is HTML', async () => {
    vi.mocked(fetch).mockResolvedValue(htmlResponse(200));
    const err = await api
      .bootstrapVoiceSession('c1', 'req-1')
      .then(() => null)
      .catch((e: unknown) => e as Error);

    expect(err).toBeInstanceOf(Error);
    expect(err?.message).not.toMatch(/Unexpected token/);
    expect(err?.message).toContain('Unexpected non-JSON response');
    expect(err?.message).toContain('HTTP 200');
    expect(err?.message).toContain('/board/');
  });
});
