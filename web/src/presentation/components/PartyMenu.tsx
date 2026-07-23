import { useEffect, useId, useRef, useState } from 'react';

export interface PartyChannelOption {
  id: string;
  name: string;
  participantCount?: number;
}

export interface PartyMenuProps {
  voiceChannels: readonly PartyChannelOption[];
  inVoiceChannelId: string | null;
  onStartParty: () => void;
  onJoinParty: (channelId: string) => void;
  onCreateParty: () => void;
  onLeaveParty?: () => void;
}

export function PartyMenu({
  voiceChannels,
  inVoiceChannelId,
  onStartParty,
  onJoinParty,
  onCreateParty,
  onLeaveParty,
}: PartyMenuProps) {
  const [open, setOpen] = useState(false);
  const [showJoinList, setShowJoinList] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const menuId = useId();

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: PointerEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
        setShowJoinList(false);
      }
    };
    document.addEventListener('pointerdown', onPointerDown);
    return () => document.removeEventListener('pointerdown', onPointerDown);
  }, [open]);

  const close = () => {
    setOpen(false);
    setShowJoinList(false);
  };

  return (
    <div className="party-menu" ref={rootRef}>
      <button
        type="button"
        className={`party-menu__trigger${inVoiceChannelId ? ' party-menu__trigger--active' : ''}`}
        aria-expanded={open}
        aria-haspopup="menu"
        aria-controls={menuId}
        onClick={() => setOpen((value) => !value)}
      >
        Party
      </button>
      {open && (
        <div className="party-menu__panel" id={menuId} role="menu">
          {inVoiceChannelId && onLeaveParty ? (
            <button
              type="button"
              className="party-menu__item"
              role="menuitem"
              onClick={() => {
                onLeaveParty();
                close();
              }}
            >
              Leave Party
            </button>
          ) : (
            <>
              <button
                type="button"
                className="party-menu__item"
                role="menuitem"
                onClick={() => {
                  onStartParty();
                  close();
                }}
              >
                Start Party
              </button>
              <button
                type="button"
                className="party-menu__item"
                role="menuitem"
                aria-expanded={showJoinList}
                onClick={() => setShowJoinList((value) => !value)}
              >
                Join Party
              </button>
              {showJoinList && (
                <ul className="party-menu__sublist">
                  {voiceChannels.length === 0 ? (
                    <li className="party-menu__empty">No parties yet — start or create one.</li>
                  ) : (
                    voiceChannels.map((channel) => (
                      <li key={channel.id}>
                        <button
                          type="button"
                          className="party-menu__subitem"
                          role="menuitem"
                          onClick={() => {
                            onJoinParty(channel.id);
                            close();
                          }}
                        >
                          <span>{channel.name}</span>
                          {channel.participantCount != null && (
                            <span className="party-menu__count">{channel.participantCount}</span>
                          )}
                        </button>
                      </li>
                    ))
                  )}
                </ul>
              )}
              <button
                type="button"
                className="party-menu__item"
                role="menuitem"
                onClick={() => {
                  onCreateParty();
                  close();
                }}
              >
                Create new Party
              </button>
            </>
          )}
          <p className="party-menu__hint">Voice is in preview — quality may vary.</p>
        </div>
      )}
    </div>
  );
}
