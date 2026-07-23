import { useCallback, useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { api } from '../../api/client';
import type { Server } from '../../api/types';
import { clearLastChannelId } from '../utils/server-last-channel';
import { ServerCard } from './ServerCard';
import { ServerChannelNav } from './ServerChannelNav';
import { ModalPortal } from './ModalPortal';

const HARBOUR_HOME_ID = '00000000-0000-4000-8000-000000000001';

export function ServerSidebar() {
  const navigate = useNavigate();
  const { serverId: activeServerId } = useParams<{ serverId?: string }>();
  const [servers, setServers] = useState<Server[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreate, setShowCreate] = useState(false);
  const [editServer, setEditServer] = useState<Server | null>(null);
  const [formName, setFormName] = useState('');
  const [formDescription, setFormDescription] = useState('');
  const [formIconUrl, setFormIconUrl] = useState('');
  const [formCardColor, setFormCardColor] = useState('#2f8f7f');
  const [saving, setSaving] = useState(false);

  const loadServers = useCallback(async () => {
    const list = await api.listServers();
    setServers(list);
  }, []);

  useEffect(() => {
    void (async () => {
      try {
        await loadServers();
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to load servers');
      } finally {
        setLoading(false);
      }
    })();
  }, [loadServers]);

  const canManageServer = useCallback((server: Server) => {
    if (server.id === HARBOUR_HOME_ID) return false;
    return server.myRole === 'owner' || server.myRole === 'admin';
  }, []);

  const openCreate = () => {
    setFormName('');
    setFormDescription('');
    setShowCreate(true);
  };

  const openEdit = (server: Server) => {
    setFormName(server.name);
    setFormDescription(server.description ?? '');
    setFormIconUrl(server.icon_url ?? '');
    setFormCardColor(server.cardColor ?? '#2f8f7f');
    setEditServer(server);
  };

  const openServer = (server: Server) => {
    navigate(`/servers/${server.id}`, { state: { openLastChannel: true } });
  };

  const createServer = async () => {
    if (!formName.trim()) return;
    setSaving(true);
    setError(null);
    try {
      const created = await api.createServer(formName.trim(), formDescription.trim() || undefined);
      setShowCreate(false);
      await loadServers();
      navigate(`/servers/${created.id}`, { state: { openLastChannel: true } });
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Create failed');
    } finally {
      setSaving(false);
    }
  };

  const saveEdit = async () => {
    if (!editServer || !formName.trim()) return;
    setSaving(true);
    setError(null);
    try {
      await api.updateServer(editServer.id, {
        name: formName.trim(),
        description: formDescription.trim(),
        iconUrl: formIconUrl.trim(),
        cardColor: formCardColor.trim(),
      });
      setEditServer(null);
      await loadServers();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Update failed');
    } finally {
      setSaving(false);
    }
  };

  const removeServer = async (server: Server) => {
    if (server.id === HARBOUR_HOME_ID) return;
    if (!window.confirm(`Remove "${server.name}" and all its channels?`)) return;
    setError(null);
    try {
      await api.deleteServer(server.id);
      clearLastChannelId(server.id);
      if (activeServerId === server.id) navigate('/servers');
      await loadServers();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Remove failed');
    }
  };

  return (
    <aside className={`servers-sidebar${activeServerId ? ' servers-sidebar--open' : ''}`}>
      <div className="servers-sidebar__stage">
        <div className="servers-sidebar__pane servers-sidebar__pane--list">
          <header className="servers-sidebar__header">
            <h1>Your Servers</h1>
            <button type="button" className="servers-sidebar__add" aria-label="Create server" onClick={openCreate}>
              +
            </button>
          </header>
          <div className="servers-sidebar__list">
            {loading ? (
              <p className="board-page__hint">Loading…</p>
            ) : servers.length === 0 ? (
              <p className="board-page__hint">No servers yet. Tap + to create one.</p>
            ) : (
              servers.map((server) => (
                <ServerCard
                  key={server.id}
                  server={server}
                  active={server.id === activeServerId}
                  canManage={canManageServer(server)}
                  onOpen={() => openServer(server)}
                  onEdit={() => openEdit(server)}
                  onDelete={() => void removeServer(server)}
                />
              ))
            )}
          </div>
          {error && (
            <p className="servers-sidebar__error" role="alert">
              {error}
            </p>
          )}
        </div>

        <div className="servers-sidebar__pane servers-sidebar__pane--channels">
          {activeServerId && <ServerChannelNav />}
        </div>
      </div>

      {showCreate && (
        <ModalPortal>
          <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
          <form
            className="chat-modal"
            onSubmit={(e) => {
              e.preventDefault();
              void createServer();
            }}
          >
            <h2>New server</h2>
            <input
              value={formName}
              onChange={(e) => setFormName(e.target.value)}
              placeholder="Server name"
              autoFocus
            />
            <input
              value={formDescription}
              onChange={(e) => setFormDescription(e.target.value)}
              placeholder="Description (optional)"
            />
            <div className="chat-modal__actions">
              <button type="button" onClick={() => setShowCreate(false)}>
                Cancel
              </button>
              <button type="submit" disabled={saving || !formName.trim()}>
                Create
              </button>
            </div>
          </form>
        </div>
        </ModalPortal>
      )}

      {editServer && (
        <ModalPortal>
          <div className="chat-modal-backdrop" role="dialog" aria-modal="true">
          <form
            className="chat-modal"
            onSubmit={(e) => {
              e.preventDefault();
              void saveEdit();
            }}
          >
            <h2>Edit server</h2>
            <input
              value={formName}
              onChange={(e) => setFormName(e.target.value)}
              placeholder="Server name"
              autoFocus
            />
            <input
              value={formDescription}
              onChange={(e) => setFormDescription(e.target.value)}
              placeholder="Description"
            />
            <label className="chat-modal__field">
              <span>Avatar image URL</span>
              <input value={formIconUrl} onChange={(e) => setFormIconUrl(e.target.value)} placeholder="https://…" />
            </label>
            <label className="chat-modal__field">
              <span>Card colour</span>
              <input
                type="color"
                value={formCardColor}
                onChange={(e) => setFormCardColor(e.target.value)}
              />
            </label>
            <div className="chat-modal__actions">
              <button type="button" onClick={() => setEditServer(null)}>
                Cancel
              </button>
              <button type="submit" disabled={saving || !formName.trim()}>
                Save
              </button>
            </div>
          </form>
        </div>
        </ModalPortal>
      )}
    </aside>
  );
}
