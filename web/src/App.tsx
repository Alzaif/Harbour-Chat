import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { BoardShell } from './presentation/shell/BoardShell';
import { ServersShell } from './presentation/shell/ServersShell';
import { BoardFeedPage } from './presentation/pages/BoardFeedPage';
import { PostDetailPage } from './presentation/pages/PostDetailPage';
import { DirectPage } from './presentation/pages/DirectPage';
import { GroupsPage } from './presentation/pages/GroupsPage';
import { ServersHome } from './presentation/pages/ServersHome';
import { LegacyGroupsRedirect } from './presentation/shell/LegacyGroupsRedirect';

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<BoardShell />}>
        <Route index element={<Navigate to="direct" replace />} />
        <Route path="feed" element={<BoardFeedPage />} />
        <Route path="feed/:postId" element={<PostDetailPage />} />
        <Route path="direct" element={<DirectPage />} />
        <Route path="direct/:channelId" element={<DirectPage />} />
        <Route path="servers" element={<ServersShell />}>
          <Route index element={<ServersHome />} />
          <Route path=":serverId" element={<GroupsPage />} />
          <Route path=":serverId/:channelId" element={<GroupsPage />} />
        </Route>
        <Route path="groups/*" element={<LegacyGroupsRedirect />} />
      </Route>
      <Route path="*" element={<Navigate to="/direct" replace />} />
    </Routes>
  );
}

export function App() {
  const basename = import.meta.env.BASE_URL.replace(/\/$/, '') || '/board';
  return (
    <BrowserRouter basename={basename}>
      <AppRoutes />
    </BrowserRouter>
  );
}
