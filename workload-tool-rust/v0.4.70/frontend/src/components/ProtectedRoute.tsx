import React, { type ReactNode } from 'react';
import { Navigate } from 'react-router-dom';
import { useUser } from '../UserContext';
import { hasAnyPrefix } from '../constants/permissions';

interface ProtectedRouteProps {
  children: ReactNode;
  requireAdmin?: boolean;
  /** 需要任一 manage:* 管理权限（管理员或拥有 manage:* 权限的用户均可进入） */
  requireManage?: boolean;
}

const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children, requireAdmin, requireManage }) => {
  const { isLoggedIn, user } = useUser();
  if (!isLoggedIn) return <Navigate to="/login" replace />;
  if (requireAdmin && !user?.is_admin) return <Navigate to="/" replace />;
  if (requireManage && !(user?.is_admin || hasAnyPrefix(user?.permissions || [], 'manage:'))) {
    return <Navigate to="/" replace />;
  }
  return <>{children}</>;
};

export default ProtectedRoute;
