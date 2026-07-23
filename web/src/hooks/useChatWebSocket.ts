import { useEffect, useRef } from 'react';
import type { Message } from '../api/types';
import { apiUrl } from '../api/app-path';

type WsPayload =
  | { type: 'post_created'; post: import('../api/types').Post }
  | { type: 'message_created'; message: Message }
  | { type: 'message_updated'; message: Message }
  | { type: 'message_deleted'; message_id: string; channel_id: string }
  | { type: 'reaction_updated'; message_id: string; channel_id: string }
  | {
      type: 'typing_started';
      channel_id: string;
      user_id: string;
      display_name?: string | null;
      expires_at: string;
    }
  | { type: 'typing_stopped'; channel_id: string; user_id: string }
  | {
      type: 'presence_changed';
      server_id: string;
      user_id: string;
      status: 'online' | 'idle' | 'dnd' | 'offline';
      updated_at: string;
    }
  | {
      type: 'voice_participant_updated';
      channel_id: string;
      user_id: string;
      display_name?: string | null;
      connected: boolean;
      muted: boolean;
      deafened: boolean;
      updated_at: string;
    }
  | {
      type: 'signal_response';
      request_id: string;
      kind: string;
      ok: boolean;
      payload?: unknown;
      error?: string;
    };

export function useChatWebSocket(
  channelIds: readonly string[],
  onEvent: (payload: WsPayload) => void,
) {
  const onEventRef = useRef(onEvent);
  onEventRef.current = onEvent;

  useEffect(() => {
    if (channelIds.length === 0) return;
    if (typeof window === 'undefined' || typeof WebSocket === 'undefined') return;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    let disposed = false;
    let ws: WebSocket | null = null;
    let reconnectTimer: number | null = null;
    let reconnectAttempt = 0;

    const scheduleReconnect = () => {
      if (disposed) return;
      reconnectAttempt += 1;
      const delayMs = Math.min(1000 * 2 ** Math.min(reconnectAttempt, 4), 10000);
      reconnectTimer = window.setTimeout(connect, delayMs);
    };

    const connect = () => {
      if (disposed) return;
      const wsUrl = `${protocol}//${window.location.host}${apiUrl('/api/ws')}`;
      ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        reconnectAttempt = 0;
        try {
          ws?.send(JSON.stringify({ type: 'subscribe', channel_ids: [...channelIds] }));
        } catch {
          /* ignore subscribe failures; reconnect will retry */
        }
      };

      ws.onclose = () => {
        scheduleReconnect();
      };

      ws.onerror = () => {
        ws?.close();
      };

      ws.onmessage = (ev) => {
        try {
          const data = JSON.parse(ev.data as string) as {
            type: string;
            message?: Message;
            message_id?: string;
            channel_id?: string;
            user_id?: string;
            display_name?: string | null;
            expires_at?: string;
            server_id?: string;
            status?: 'online' | 'idle' | 'dnd' | 'offline';
            updated_at?: string;
            connected?: boolean;
            muted?: boolean;
            deafened?: boolean;
          request_id?: string;
          kind?: string;
          ok?: boolean;
          payload?: unknown;
          error?: string;
          };
          if (data.type === 'message_created' && data.message) {
            onEventRef.current({ type: 'message_created', message: data.message });
          } else if (data.type === 'message_updated' && data.message) {
            onEventRef.current({ type: 'message_updated', message: data.message });
          } else if (data.type === 'message_deleted' && data.message_id && data.channel_id) {
            onEventRef.current({
              type: 'message_deleted',
              message_id: data.message_id,
              channel_id: data.channel_id,
            });
          } else if (data.type === 'reaction_updated' && data.message_id && data.channel_id) {
            onEventRef.current({
              type: 'reaction_updated',
              message_id: data.message_id,
              channel_id: data.channel_id,
            });
          } else if (
            data.type === 'typing_started' &&
            data.channel_id &&
            data.user_id &&
            data.expires_at
          ) {
            onEventRef.current({
              type: 'typing_started',
              channel_id: data.channel_id,
              user_id: data.user_id,
              display_name: data.display_name,
              expires_at: data.expires_at,
            });
          } else if (data.type === 'typing_stopped' && data.channel_id && data.user_id) {
            onEventRef.current({
              type: 'typing_stopped',
              channel_id: data.channel_id,
              user_id: data.user_id,
            });
          } else if (
            data.type === 'presence_changed' &&
            data.server_id &&
            data.user_id &&
            data.status &&
            data.updated_at
          ) {
            onEventRef.current({
              type: 'presence_changed',
              server_id: data.server_id,
              user_id: data.user_id,
              status: data.status,
              updated_at: data.updated_at,
            });
          } else if (
            data.type === 'voice_participant_updated' &&
            data.channel_id &&
            data.user_id &&
            typeof data.connected === 'boolean' &&
            typeof data.muted === 'boolean' &&
            typeof data.deafened === 'boolean' &&
            data.updated_at
          ) {
            onEventRef.current({
              type: 'voice_participant_updated',
              channel_id: data.channel_id,
              user_id: data.user_id,
              display_name: data.display_name,
              connected: data.connected,
              muted: data.muted,
              deafened: data.deafened,
              updated_at: data.updated_at,
            });
        } else if (
          data.type === 'signal_response' &&
          data.request_id &&
          data.kind &&
          typeof data.ok === 'boolean'
        ) {
          onEventRef.current({
            type: 'signal_response',
            request_id: data.request_id,
            kind: data.kind,
            ok: data.ok,
            payload: data.payload,
            error: data.error,
          });
          }
        } catch {
          /* ignore malformed frames */
        }
      };
    };

    connect();

    return () => {
      disposed = true;
      if (reconnectTimer) window.clearTimeout(reconnectTimer);
      ws?.close();
    };
  }, [channelIds.join(',')]);
}
