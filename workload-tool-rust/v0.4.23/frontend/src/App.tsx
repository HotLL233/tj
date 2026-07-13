import React, { type ReactNode } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { Box, CircularProgress, Typography } from '@mui/material';
import { useAuth } from './context/AuthContext';
import Layout from './components/Layout';
import LoginPage from './pages/LoginPage';
import DashboardPage from './pages/DashboardPage';
import InstrumentPage from './pages/InstrumentPage';
import InventoryPage from './pages/InventoryPage';
import PurchasePage from './pages/PurchasePage';
import ApprovalCenterPage from './pages/ApprovalCenterPage';
import NotificationsPage from './pages/NotificationsPage';
import AuditPage from './pages/AuditPage';
import AdminUsersPage from './pages/AdminUsersPage';
import AdminRolesPage from './pages/AdminRolesPage';
import AdminRulesPage from './pages/AdminRulesPage';
import ProfilePage from './pages/ProfilePage';
// 复用既有工作量统计 / 研发送样页面（路线 C：混合本地 LIMS）
import StatsPage from './pages/StatsPage';
import SamplePortal from './pages/SamplePortal';

const RequireAuth: React.FC<{ children: ReactNode }> = ({ children }) => {
  const { token, user, loading } = useAuth();
  if (loading) {
    return <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}><CircularProgress /></Box>;
  }
  if (!token || !user) return <Navigate to="/login" replace />;
  return <>{children}</>;
};

const NotFoundPage: React.FC = () => (
  <Box sx={{ p: 4, textAlign: 'center', mt: '10vh' }}>
    <Typography variant="h1" sx={{ fontSize: '4rem', color: '#ccc', m: 0 }}>404</Typography>
    <Typography color="text.secondary">页面未找到</Typography>
  </Box>
);

const App: React.FC = () => (
  <Routes>
    <Route path="/login" element={<LoginPage />} />
    <Route element={<RequireAuth><Layout /></RequireAuth>}>
      <Route path="/" element={<DashboardPage />} />
      <Route path="/instruments" element={<InstrumentPage />} />
      <Route path="/inventory" element={<InventoryPage />} />
      <Route path="/purchase" element={<PurchasePage />} />
      <Route path="/approval" element={<ApprovalCenterPage />} />
      <Route path="/notifications" element={<NotificationsPage />} />
      <Route path="/audit" element={<AuditPage />} />
      <Route path="/admin/users" element={<AdminUsersPage />} />
      <Route path="/admin/roles" element={<AdminRolesPage />} />
      <Route path="/admin/rules" element={<AdminRulesPage />} />
      <Route path="/profile" element={<ProfilePage />} />
      {/* 复用既有统计模块 */}
      <Route path="/workload" element={<StatsPage />} />
      <Route path="/sample" element={<SamplePortal />} />
      <Route path="/404" element={<NotFoundPage />} />
      <Route path="*" element={<Navigate to="/404" replace />} />
    </Route>
  </Routes>
);

export default App;
