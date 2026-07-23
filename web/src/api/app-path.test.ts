import { describe, expect, it } from 'vitest';
import { apiUrl } from './app-path';

describe('apiUrl', () => {
  it('prefixes API paths with the Vite base', () => {
    expect(apiUrl('/api/ws')).toBe('/board/api/ws');
  });
});
