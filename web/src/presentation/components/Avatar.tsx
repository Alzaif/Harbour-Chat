import { useEffect, useState } from 'react';
import { userAvatarUrl } from '../../api/app-path';
import { colorFromName, initialsFromName } from '../utils/avatar';

export interface AvatarProps {
  /** User whose avatar to load. When omitted, initials are always shown. */
  readonly userId?: string | null;
  /** Display name used for initials and the fallback color. */
  readonly name: string;
  /** Cache-busting version (e.g. avatarUpdatedAt) for the current user. */
  readonly version?: number | null;
  /** Extra class names appended to the base `chat-avatar` element. */
  readonly className?: string;
}

/**
 * Renders a user's uploaded avatar image, falling back to colored initials when
 * there is no user id or the image fails to load (i.e. no avatar uploaded).
 */
export function Avatar({ userId, name, version, className }: AvatarProps) {
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    setFailed(false);
  }, [userId, version]);

  const showImage = Boolean(userId) && !failed;
  const classes = `chat-avatar${className ? ` ${className}` : ''}`;

  return (
    <span
      className={classes}
      aria-hidden
      style={showImage ? undefined : { background: colorFromName(name) }}
    >
      {showImage ? (
        <img
          className="chat-avatar__img"
          src={userAvatarUrl(userId!, version)}
          alt=""
          onError={() => setFailed(true)}
        />
      ) : (
        initialsFromName(name)
      )}
    </span>
  );
}
