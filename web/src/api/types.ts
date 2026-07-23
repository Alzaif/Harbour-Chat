export interface CurrentUser {
  id: string;
  email: string;
  displayName: string | null;
  avatarUpdatedAt?: number | null;
}

export interface AvatarUploadResult {
  mimeType: string;
  sizeBytes: number;
  avatarUpdatedAt: number;
}

export interface Server {
  id: string;
  name: string;
  description: string | null;
  icon_url: string | null;
  cardColor: string | null;
  owner_user_id: string;
  myRole?: 'owner' | 'admin' | 'member';
}

export interface UserSettings {
  pushToTalk: boolean;
  pushToTalkKey: string;
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

export interface ReplyPreview {
  id: string;
  author_user_id: string;
  author_display_name: string | null;
  content: string;
  deleted_at: string | null;
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
  reply_to_message_id?: string | null;
  reply_to?: ReplyPreview | null;
  reactions?: ReactionSummary[];
  attachment?: MessageAttachment | null;
}

export interface Member {
  server_id: string;
  user_id: string;
  role: string;
  display_name?: string | null;
}

export interface DmInboxEntry {
  channelId: string;
  otherUserId: string;
  otherDisplayName: string | null;
  lastMessagePreview: string | null;
  unreadCount: number;
  updatedAt: number;
}

export interface DmPeer {
  id: string;
  email: string;
  displayName: string | null;
}

export type FeedPeriod = 'hour' | 'day' | 'week' | 'month' | 'year' | 'all';

export type VoteValue = -1 | 0 | 1;

export interface Post {
  id: string;
  authorUserId: string;
  authorDisplayName: string | null;
  title: string | null;
  body: string;
  linkUrl: string | null;
  previewTitle: string | null;
  previewDescription: string | null;
  previewImageUrl: string | null;
  previewSiteName: string | null;
  upvotes: number;
  downvotes: number;
  score: number;
  commentCount: number;
  myVote: VoteValue;
  createdAt: string;
  updatedAt: string;
}

export type FeedSectionKind = 'top' | 'older';

export interface FeedSection {
  kind: FeedSectionKind;
  label?: string;
  posts: Post[];
}

export interface BoardFeed {
  period: FeedPeriod;
  sections: FeedSection[];
}

export interface PostComment {
  id: string;
  postId: string;
  authorUserId: string;
  authorDisplayName: string | null;
  parentCommentId: string | null;
  body: string;
  createdAt: string;
  editedAt: string | null;
  deletedAt: string | null;
  replies: PostComment[];
}

export interface ShareTarget {
  channelId: string;
  label: string;
  kind: 'dm' | 'channel';
  serverName: string | null;
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
