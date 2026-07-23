import { cleanup, fireEvent, render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { Avatar } from './Avatar';

afterEach(() => cleanup());

describe('Avatar', () => {
  it('shows an image when a userId is provided', () => {
    const { container } = render(<Avatar userId="user-a" name="Alice" />);
    const img = container.querySelector('img.chat-avatar__img');
    expect(img).not.toBeNull();
    expect(img?.getAttribute('src')).toContain('/api/users/user-a/avatar');
  });

  it('appends a cache-busting version when provided', () => {
    const { container } = render(<Avatar userId="user-a" name="Alice" version={99} />);
    expect(container.querySelector('img')?.getAttribute('src')).toContain('v=99');
  });

  it('renders initials when there is no userId', () => {
    const { container } = render(<Avatar name="Alice Smith" />);
    expect(container.querySelector('img')).toBeNull();
    expect(container.textContent).toBe('AS');
  });

  it('falls back to initials when the image fails to load', () => {
    const { container } = render(<Avatar userId="user-a" name="Bob" />);
    const img = container.querySelector('img');
    expect(img).not.toBeNull();
    fireEvent.error(img!);
    expect(container.querySelector('img')).toBeNull();
    expect(container.textContent).toBe('BO');
  });
});
