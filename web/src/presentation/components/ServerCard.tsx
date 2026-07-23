import type { CSSProperties } from 'react';
import type { Server } from '../../api/types';
import { colorFromName, initialsFromName } from '../utils/avatar';

export function serverCardStyle(server: Server): CSSProperties {
  const bg = server.cardColor ?? colorFromName(server.name);
  const light = isLightColor(bg);
  return {
    background: bg,
    color: light ? '#1e1f22' : '#ffffff',
  };
}

function isLightColor(hex: string): boolean {
  const normalized = hex.replace('#', '');
  if (normalized.length !== 6) return false;
  const r = Number.parseInt(normalized.slice(0, 2), 16);
  const g = Number.parseInt(normalized.slice(2, 4), 16);
  const b = Number.parseInt(normalized.slice(4, 6), 16);
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return luminance > 0.62;
}

export interface ServerCardProps {
  server: Server;
  active?: boolean;
  canManage?: boolean;
  onOpen: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
}

export function ServerCard({
  server,
  active = false,
  canManage = false,
  onOpen,
  onEdit,
  onDelete,
}: ServerCardProps) {
  const style = serverCardStyle(server);

  return (
    <div className={`server-card${active ? ' server-card--active' : ''}`} style={style}>
      <button type="button" className="server-card__open" onClick={onOpen}>
        <span className="server-card__avatar" aria-hidden>
          {server.icon_url ? (
            <img src={server.icon_url} alt="" className="server-card__avatar-img" />
          ) : (
            <span style={{ background: colorFromName(server.name) }}>
              {initialsFromName(server.name)}
            </span>
          )}
        </span>
        <span className="server-card__body">
          <span className="server-card__name">{server.name}</span>
          <span className="server-card__description">
            {server.description ?? 'Open channels and topics'}
          </span>
        </span>
        <span className="server-card__cta" aria-hidden>
          Open →
        </span>
      </button>
      {canManage && (
        <div className="server-card__admin">
          {onEdit && (
            <button type="button" className="server-card__admin-btn" onClick={onEdit}>
              Edit
            </button>
          )}
          {onDelete && (
            <button type="button" className="server-card__admin-btn" onClick={onDelete}>
              Remove
            </button>
          )}
        </div>
      )}
    </div>
  );
}
