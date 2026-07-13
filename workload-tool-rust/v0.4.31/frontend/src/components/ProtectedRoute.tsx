import React, { type ReactNode } from 'react';
import { Navigate } from 'react-router-dom';
import { useUser } from '../UserContext';

interface ProtectedRouteProps {
  children: ReactNode;
  requireAdmin?: boolean;
}

const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children, requireAdmin }) => {
  const { isLoggedIn, user } = useUser();
  if (!isLoggedIn) return <Navigate to="/login" replace />;
  if (requireAdmin && !user?.is_admin) return <Navigate to="/" replace />;
  return <>{children}</>;
};

export default ProtectedRoute;
