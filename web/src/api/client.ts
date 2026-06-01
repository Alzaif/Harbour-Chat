import type {
  Channel,
  CurrentUser,
  Member,
  Message,
  PresenceState,
  PresenceStatus,
  Server,
  ServerDetail,
  TypingIndicator,
  VoiceConsumer,
  VoiceParticipant,
  VoiceProducer,
  VoiceRemoteProducer,
  VoiceSessionBootstrap,
  VoiceSignalResponse,
  VoiceTransport,
} from './types';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, {
    ...init,
    headers: {
      ...(init?.body instanceof FormData ? {} : { 'Content-Type': 'application/json' }),
      ...(init?.headers ?? {}),
    },
  });
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as { error?: string };
    throw new Error(body.error ?? `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export const api = {
  me: () => request<CurrentUser>('/api/me'),
  listServers: () => request<Server[]>('/api/servers'),
  createServer: (name: string) =>
    request<Server>('/api/servers', { method: 'POST', body: JSON.stringify({ name }) }),
  getServer: (id: string) => request<ServerDetail>(`/api/servers/${id}`),
  createChannel: (serverId: string, name: string, type: 'text' | 'voice' = 'text') =>
    request<Channel>(`/api/servers/${serverId}/channels`, {
      method: 'POST',
      body: JSON.stringify({ name, type }),
    }),
  listMembers: (serverId: string) => request<Member[]>(`/api/servers/${serverId}/members`),
  listMessages: (channelId: string, before?: string) => {
    const params = new URLSearchParams({ limit: '50' });
    if (before) params.set('before', before);
    return request<Message[]>(`/api/channels/${channelId}/messages?${params}`);
  },
  sendMessage: (channelId: string, content: string) =>
    request<Message>(`/api/channels/${channelId}/messages`, {
      method: 'POST',
      body: JSON.stringify({ content }),
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
