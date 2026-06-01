export interface CurrentUser {
  id: string;
  email: string;
  displayName: string | null;
}

export interface Server {
  id: string;
  name: string;
  icon_url: string | null;
  owner_user_id: string;
}

export interface Channel {
  id: string;
  server_id: string | null;
  type: 'text' | 'voice' | 'dm';
  name: string;
  position: number;
}

export interface ServerDetail {
  server: Server;
  channels: Channel[];
  unreadByChannelId: Record<string, number>;
}

export interface ReactionSummary {
  emoji: string;
  count: number;
  userIds: string[];
}

export interface MessageAttachment {
  id: string;
  filename: string;
  mime_type: string;
  size_bytes: number;
}

export interface Message {
  id: string;
  channel_id: string;
  author_user_id: string;
  author_display_name: string | null;
  content: string;
  created_at: string;
  edited_at: string | null;
  deleted_at: string | null;
  reactions?: ReactionSummary[];
  attachment?: MessageAttachment | null;
}

export interface Member {
  server_id: string;
  user_id: string;
  role: string;
  display_name?: string | null;
}

export type PresenceStatus = 'online' | 'idle' | 'dnd' | 'offline';

export interface PresenceState {
  server_id: string;
  user_id: string;
  status: PresenceStatus;
  updated_at: string;
}

export interface TypingIndicator {
  channel_id: string;
  user_id: string;
  display_name?: string | null;
  expires_at: string;
}

export interface VoiceParticipant {
  channel_id: string;
  user_id: string;
  display_name?: string | null;
  muted: boolean;
  deafened: boolean;
  updated_at: string;
}

export interface IceServer {
  urls: string[];
  username?: string;
  credential?: string;
}

export interface VoiceSessionBootstrap {
  session_id: string;
  channel_id: string;
  user_id: string;
  sfu_base_url: string;
  router_rtp_capabilities: unknown;
  ice_servers: IceServer[];
  expires_at: string;
}

export interface VoiceTransport {
  session_id: string;
  transport_id: string;
  direction: string;
  ice_parameters: unknown;
  ice_candidates: unknown;
  dtls_parameters: unknown;
}

export interface VoiceProducer {
  session_id: string;
  producer_id: string;
  transport_id: string;
  kind: string;
}

export interface VoiceConsumer {
  session_id: string;
  consumer_id: string;
  producer_id: string;
  transport_id: string;
  kind: string;
  rtp_parameters: unknown;
}

export interface VoiceRemoteProducer {
  producer_id: string;
  kind: string;
  user_id: string;
}

export interface VoiceSignalResponse<T> {
  type: 'signal_response';
  requestId: string;
  kind: string;
  ok: boolean;
  payload: T;
}
