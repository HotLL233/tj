// 兼容层：旧的工作量统计页面仍从 './UserContext' 引用 useUser / UserProvider，
// 现统一委托给 AuthContext（鉴权 + RBAC）。
export { useAuth as useUser, AuthProvider as UserProvider } from './context/AuthContext';
