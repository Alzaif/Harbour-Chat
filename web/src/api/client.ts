import type {
  AvatarUploadResult,
  BoardFeed,
  Channel,
  CurrentUser,
  DmInboxEntry,
  DmPeer,
  FeedPeriod,
  Member,
  Message,
  Post,
  PostComment,
  ShareTarget,
  UserSettings,
  PresenceState,
  PresenceStatus,
  Server,
  ServerDetail,
  TypingIndicator,
  VoteValue,
  VoiceConsumer,
  VoiceParticipant,
  VoiceProducer,
  VoiceRemoteProducer,
  VoiceSessionBootstrap,
  VoiceSignalResponse,
  VoiceTransport,
} from './types';
import { apiUrl } from './app-path';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const url = apiUrl(path);
  const res = await fetch(url, {
    ...init,
    credentials: 'same-origin',
    headers: {
      ...(init?.body instanceof FormData ? {} : { 'Content-Type': 'application/json' }),
      ...(init?.headers ?? {}),
    },
  });

  const isJson = (res.headers.get('content-type') ?? '').includes('application/json');

  if (!res.ok) {
    let message = `HTTP ${res.status}`;
    if (isJson) {
      const body = (await res.json().catch(() => null)) as { error?: string } | null;
      if (body?.error) message = body.error;
    }
    throw new Error(message);
  }

  if (res.status === 204) return undefined as T;

  // A 2xx response that isn't JSON almost always means the request was routed
  // to the platform shell or this app's SPA index instead of the API.
  if (!isJson) {
    throw new Error(
      `Unexpected non-JSON response (HTTP ${res.status}) from ${url}. ` +
        'The request may have missed the /board API route — rebuild and redeploy harbour-chat.',
    );
  }

  return (await res.json()) as T;
}

export const api = {
  me: () => request<CurrentUser>('/api/me'),
  uploadAvatar: (file: File) => {
    const form = new FormData();
    form.append('file', file);
    return request<AvatarUploadResult>('/api/me/avatar', {
      method: 'POST',
      body: form,
    });
  },
  getSettings: () => request<UserSettings>('/api/me/settings'),
  updateSettings: (settings: Partial<UserSettings>) =>
    request<UserSettings>('/api/me/settings', {
      method: 'PATCH',
      body: JSON.stringify(settings),
    }),
  listServers: () => request<Server[]>('/api/servers'),
  createServer: (name: string, description?: string) =>
    request<Server>('/api/servers', {
      method: 'POST',
      body: JSON.stringify({ name, description }),
    }),
  updateServer: (
    id: string,
    patch: { name?: string; description?: string; iconUrl?: string; cardColor?: string },
  ) =>
    request<Server>(`/api/servers/${id}`, {
      method: 'PATCH',
      body: JSON.stringify({
        name: patch.name,
        description: patch.description,
        iconUrl: patch.iconUrl,
        cardColor: patch.cardColor,
      }),
    }),
  deleteServer: (id: string) =>
    request<{ ok: boolean }>(`/api/servers/${id}`, { method: 'DELETE' }),
  getServer: (id: string) => request<ServerDetail>(`/api/servers/${id}`),
  createChannel: (serverId: string, name: string, type: 'text' | 'voice' = 'text') =>
    request<Channel>(`/api/servers/${serverId}/channels`, {
      method: 'POST',
      body: JSON.stringify({ name, type }),
    }),
  listMembers: (serverId: string) => request<Member[]>(`/api/servers/${serverId}/members`),
  addMember: (serverId: string, userId: string) =>
    request<Member>(`/api/servers/${serverId}/members`, {
      method: 'POST',
      body: JSON.stringify({ userId }),
    }),
  searchUsers: (query: string, excludeServerId?: string) => {
    const params = new URLSearchParams({ q: query });
    if (excludeServerId) params.set('excludeServerId', excludeServerId);
    return request<DmPeer[]>(`/api/users/search?${params}`);
  },
  listMessages: (channelId: string, before?: string) => {
    const params = new URLSearchParams({ limit: '50' });
    if (before) params.set('before', before);
    return request<Message[]>(`/api/channels/${channelId}/messages?${params}`);
  },
  sendMessage: (
    channelId: string,
    content: string,
    options?: { replyToMessageId?: string },
  ) =>
    request<Message>(`/api/channels/${channelId}/messages`, {
      method: 'POST',
      body: JSON.stringify({
        content,
        ...(options?.replyToMessageId
          ? { reply_to_message_id: options.replyToMessageId }
          : {}),
      }),
    }),
  toggleReaction: (messageId: string, emoji: string) =>
    request<{ added: boolean }>(`/api/messages/${messageId}/reactions`, {
      method: 'POST',
      body: JSON.stringify({ emoji }),
    }),
  markRead: (channelId: string, messageId: string) =>
    request<{ ok: boolean }>(`/api/channels/${channelId}/read`, {
      method: 'POST',
      body: JSON.stringify({ messageId }),
    }),
  uploadAttachment: (messageId: string, file: File) => {
    const form = new FormData();
    form.append('file', file);
    return request<Message>(`/api/messages/${messageId}/attachments`, {
      method: 'POST',
      body: form,
    });
  },
  openDm: (userId: string) =>
    request<Channel>(`/api/dms/${encodeURIComponent(userId)}`, { method: 'POST' }),
  listDms: () => request<DmInboxEntry[]>('/api/dms'),
  listDmPeers: () => request<DmPeer[]>('/api/dm-peers'),
  listBoardPosts: async (period: FeedPeriod = 'day', limit = 20) => {
    const params = new URLSearchParams({ period, limit: String(limit) });
    const data = await request<BoardFeed | Post[]>(`/api/board/posts?${params}`);
    if (Array.isArray(data)) {
      return { period, sections: [{ kind: 'top' as const, posts: data }] };
    }
    return {
      period: (data?.period as FeedPeriod) ?? period,
      sections: Array.isArray(data?.sections) ? data.sections : [],
    };
  },
  createBoardPost: (body: { title?: string; body: string; link_url?: string }) =>
    request<Post>('/api/board/posts', {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  getBoardPost: (id: string) => request<Post>(`/api/board/posts/${id}`),
  voteBoardPost: (id: string, value: VoteValue) =>
    request<Post>(`/api/board/posts/${id}/vote`, {
      method: 'POST',
      body: JSON.stringify({ value }),
    }),
  listBoardComments: (postId: string) =>
    request<PostComment[]>(`/api/board/posts/${postId}/comments`),
  createBoardComment: (postId: string, body: { body: string; parentCommentId?: string }) =>
    request<PostComment>(`/api/board/posts/${postId}/comments`, {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  listShareTargets: () => request<ShareTarget[]>('/api/board/share-targets'),
  shareBoardPost: (postId: string, channelId: string) =>
    request<Message>(`/api/board/posts/${postId}/share`, {
      method: 'POST',
      body: JSON.stringify({ channelId }),
    }),
  listPresence: (serverId: string) => request<PresenceState[]>(`/api/servers/${serverId}/presence`),
  setPresence: (serverId: string, status: PresenceStatus) =>
    request<PresenceState>(`/api/servers/${serverId}/presence`, {
      method: 'POST',
      body: JSON.stringify({ status }),
    }),
  listTyping: (channelId: string) => request<TypingIndicator[]>(`/api/channels/${channelId}/typing`),
  setTyping: (channelId: string, isTyping: boolean) =>
    request<TypingIndicator[]>(`/api/channels/${channelId}/typing`, {
      method: 'POST',
      body: JSON.stringify({ isTyping }),
    }),
  listVoiceParticipants: (channelId: string) =>
    request<VoiceParticipant[]>(`/api/channels/${channelId}/voice`),
  joinVoice: (channelId: string, muted = false, deafened = false) =>
    request<VoiceParticipant>(`/api/channels/${channelId}/voice/join`, {
      method: 'POST',
      body: JSON.stringify({ muted, deafened }),
    }),
  leaveVoice: (channelId: string) =>
    request<{ ok: boolean }>(`/api/channels/${channelId}/voice/leave`, {
      method: 'POST',
      body: JSON.stringify({}),
    }),
  updateVoiceState: (channelId: string, muted: boolean, deafened: boolean) =>
    request<VoiceParticipant>(`/api/channels/${channelId}/voice/state`, {
      method: 'POST',
      body: JSON.stringify({ muted, deafened }),
    }),
  bootstrapVoiceSession: (channelId: string, requestId: string) =>
    request<VoiceSignalResponse<VoiceSessionBootstrap>>(`/api/channels/${channelId}/voice/session`, {
      method: 'POST',
      body: JSON.stringify({ requestId }),
    }),
  createVoiceTransport: (
    channelId: string,
    requestId: string,
    sessionId: string,
    direction: 'send' | 'recv',
  ) =>
    request<VoiceSignalResponse<VoiceTransport>>(`/api/channels/${channelId}/voice/transports`, {
      method: 'POST',
      body: JSON.stringify({ requestId, sessionId, direction }),
    }),
  connectVoiceTransport: (
    channelId: string,
    transportId: string,
    requestId: string,
    sessionId: string,
    dtlsParameters: unknown,
  ) =>
    request<VoiceSignalResponse<{ ok: boolean }>>(
      `/api/channels/${channelId}/voice/transports/${transportId}/connect`,
      {
        method: 'POST',
        body: JSON.stringify({ requestId, sessionId, dtlsParameters }),
      },
    ),
  createVoiceProducer: (
    channelId: string,
    requestId: string,
    sessionId: string,
    transportId: string,
    kind: 'audio',
    rtpParameters: unknown,
  ) =>
    request<VoiceSignalResponse<VoiceProducer>>(`/api/channels/${channelId}/voice/producers`, {
      method: 'POST',
      body: JSON.stringify({ requestId, sessionId, transportId, kind, rtpParameters }),
    }),
  createVoiceConsumer: (
    channelId: string,
    requestId: string,
    sessionId: string,
    transportId: string,
    producerId: string,
    rtpCapabilities: unknown,
  ) =>
    request<VoiceSignalResponse<VoiceConsumer>>(`/api/channels/${channelId}/voice/consumers`, {
      method: 'POST',
      body: JSON.stringify({ requestId, sessionId, transportId, producerId, rtpCapabilities }),
    }),
  addVoiceIceCandidate: (
    channelId: string,
    transportId: string,
    requestId: string,
    sessionId: string,
    candidate: unknown,
  ) =>
    request<VoiceSignalResponse<{ ok: boolean }>>(
      `/api/channels/${channelId}/voice/transports/${transportId}/ice-candidates`,
      {
        method: 'POST',
        body: JSON.stringify({ requestId, sessionId, candidate }),
      },
    ),
  restartVoiceIce: (
    channelId: string,
    transportId: string,
    requestId: string,
    sessionId: string,
  ) =>
    request<VoiceSignalResponse<unknown>>(
      `/api/channels/${channelId}/voice/transports/${transportId}/restart-ice`,
      {
        method: 'POST',
        body: JSON.stringify({ requestId, sessionId }),
      },
    ),
  listRemoteVoiceProducers: (channelId: string, sessionId: string) =>
    request<{ producers: VoiceRemoteProducer[] }>(
      `/api/channels/${channelId}/voice/remote-producers?sessionId=${encodeURIComponent(sessionId)}`,
    ),
  closeVoiceSession: (channelId: string, sessionId: string) =>
    request<{ ok: boolean }>(
      `/api/channels/${channelId}/voice/session/${encodeURIComponent(sessionId)}`,
      { method: 'DELETE' },
    ),
};
