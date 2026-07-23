import { useCallback, useEffect, useMemo, useState } from 'react';
import { useLocation, useNavigate, useParams } from 'react-router-dom';
import { api } from '../../api/client';
import type { Channel } from '../../api/types';
import { splitGroupChannels } from '../utils/group-channels';
import {
  rememberLastChannelId,
  resolveChannelForServer,
} from '../utils/server-last-channel';
import { ModalPortal } from './ModalPortal';

type OpenLastChannelState = { openLastChannel?: boolean };

export function ServerChannelNav() {
  const navigate = useNavigate();
  const location = useLocation();
  const { serverId, channelId: activeChannelId } = useParams<{
    serverId: string;
    channelId?: string;
  }>();
  const [serverName, setServerName] = useState('Server');
  const [channels, setChannels] = useState<Channel[]>([]);
  const [unreadByChannel, setUnreadByChannel] = useState<Record<string, number>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreateTopic, setShowCreateTopic] = useState(false);
  const [newTopicName, setNewTopicName] = useState('');

  const { mainChannel, topicChannels } = useMemo(() => splitGroupChannels(channels), [channels]);
  const textChannelIds = useMemo(() => channels.map((channel) => channel.id), [channels]);

  const loadServer = useCallback(async (id: string) => {
    const detail = await api.getServer(id);
    setServerName(detail.server.name);
    setChannels(detail.channels.filter((channel) => channel.type === 'text'));
    setUnreadByChannel(detail.unreadByChannelId ?? {});
  }, []);

  useEffect(() => {
    if (!serverId) return;
    let cancelled = false;
    void (async () => {
      setLoading(true);
      setError(null);
      try {
        await loadServer(serverId);
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : 'Failed to load server');
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [serverId, loadServer]);

  useEffect(() => {
    if (!serverId || activeChannelId || loading || textChannelIds.length === 0) return;
    const navState = location.state as OpenLastChannelState | null;
    if (!navState?.openLastChannel) return;

    const target = resolveChannelForServer(serverId, textChannelIds, mainChannel?.id);
    if (!target) return;
    navigate(`/servers/${serverId}/${target}`, { replace: true, state: {} });
  }, [
    serverId,
    activeChannelId,
    loading,
    textChannelIds,
    mainChannel?.id,
    location.state,
    navigate,
  ]);

  const openChannel = (channel: Channel) => {
    if (!serverId) return;
    rememberLastChannelId(serverId, channel.id);
    navigate(`/servers/${serverId}/${channel.id}`);
  };

  const createTopic = async () => {
    if (!serverId || !newTopicName.trim()) return;
    try {
      const created = await api.createChannel(serverId, newTopicName.trim(), 'text');
      setShowCreateTopic(false);
      setNewTopicName('');
      await loadServer(serverId);
      openChannel(created);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Create topic failed');
    }
  };

  if (!serverId) return null;

  return (
    <>
      <header className="servers-channel-nav__header">
        <button
          type="button"
          className="servers-channel-nav__back"
          onClick={() => navigate('/servers')}
          aria-label="Back to servers"
        >
          ←
        </button>
        <h2 className="servers-channel-nav__title">{serverName}</h2>
      </header>

      <div className="servers-channel-nav__body">
        {loading ? (
          <p className="board-page__hint">Loading…</p>
        ) : !mainChannel ? (
          <p className="board-page__hint">This server has no channels yet.</p>
        ) : (
          <>
            <ul className="servers-channel-nav__list">
              <li>
                <button
                  type="button"
                  className={`servers-channel-nav__item${mainChannel.id === activeChannelId ? ' servers-channel-nav__item--active' : ''}`}
                  onClick={() => openChannel(mainChannel)}
                >
                  <span className="servers-channel-nav__item-name">{mainChannel.name}</span>
                  <span className="servers-channel-nav__item-label">Main</span>
                  {(unreadByChannel[mainChannel.id] ?? 0) > 0 && (
                    <span className="groups-topic-item__badge">{unreadByChannel[mainChannel.id]}</span>
                  )}
                </button>
              </li>
              {topicChannels.map((topic) => (
                <li key={topic.id}>
                  <button
                    type="button"
                    className={`servers-channel-nav__item${topic.id === activeChannelId ? ' servers-channel-nav__item--active' : ''}`}
                    onClick={() => openChannel(topic)}
                  >
                    <span className="servers-channel-nav__item-name">{topic.name}</span>
                    {(unreadByChannel[topic.id] ?? 0) > 0 && (
                      <span className="groups-topic-item__badge">{unreadByChannel[topic.id]}</span>
                    )}
                  </button>
                </li>
              ))}
            </ul>
            <button type="button" className="groups-create-topic" onClick={() => setShowCreateTopic(true)}>
              + New topic
            </button>
          </>
        )}
      </div>

      {error && (
        <p className="servers-sidebar__error" role="alert">
          {error}
        </p>
      )}

      {showCreateTopic && (
        <ModalPortal>
          <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
            <form
              className="chat-modal"
              onSubmit={(e) => {
                e.preventDefault();
                void createTopic();
              }}
            >
              <h2>New topic</h2>
              <input
                value={newTopicName}
                onChange={(e) => setNewTopicName(e.target.value)}
                placeholder="Topic name"
                autoFocus
              />
              <div className="chat-modal__actions">
                <button type="button" onClick={() => setShowCreateTopic(false)}>
                  Cancel
                </button>
                <button type="submit">Create</button>
              </div>
            </form>
          </div>
        </ModalPortal>
      )}
    </>
  );
}
