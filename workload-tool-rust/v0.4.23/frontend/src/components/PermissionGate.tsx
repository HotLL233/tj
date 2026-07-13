import React, { type ReactNode } from 'react';
import { useAuth } from '../context/AuthContext';

interface PermissionGateProps {
  /** 需要的权限点（任一满足即可；含 `*` 通配符时恒通过） */
  perm?: string;
  children: ReactNode;
  /** 无权限时渲染的内容（默认不渲染） */
  fallback?: ReactNode;
}

/** 基于当前用户权限渲染子内容。 */
const PermissionGate: React.FC<PermissionGateProps> = ({ perm, children, fallback = null }) => {
  const { hasPerm } = useAuth();
  if (perm && !hasPerm(perm)) return <>{fallback}</>;
  return <>{children}</>;
};

export default PermissionGate;
