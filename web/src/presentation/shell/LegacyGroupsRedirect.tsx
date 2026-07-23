import { Navigate, useLocation } from 'react-router-dom';

export function LegacyGroupsRedirect() {
  const location = useLocation();
  const target = location.pathname.replace(/^\/groups(?=\/|$)/, '/servers');
  return <Navigate to={`${target}${location.search}${location.hash}`} replace />;
}
