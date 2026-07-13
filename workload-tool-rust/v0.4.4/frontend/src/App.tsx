import React, { Component, type ErrorInfo, type ReactNode } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { UserProvider } from './UserContext';
import Layout from './components/Layout';
import HomePage from './pages/HomePage';
import SamplePortal from './pages/SamplePortal';
import WorkloadPortal from './pages/WorkloadPortal';
import EntryPage from './pages/EntryPage';
import SampleEntryPage from './pages/SampleEntryPage';
import SampleStatsPage from './pages/SampleStatsPage';
import RdRecordsPage from './pages/RdRecordsPage';
import StatsPage from './pages/StatsPage';
import ManagePage from './pages/ManagePage';

interface ErrorBoundaryState { hasError: boolean; error: Error | null; }
class ErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false, error: null };
  static getDerivedStateFromError(error: Error): ErrorBoundaryState { return { hasError: true, error }; }
  componentDidCatch(error: Error, info: ErrorInfo) { console.error('ErrorBoundary caught error:', error, info); }
  render() { if (this.state.hasError) return <div style={{ padding: '2rem', textAlign: 'center', marginTop: '20vh' }}><h1>页面出错了</h1><p style={{ color: '#666', margin: '1rem 0' }}>{this.state.error?.message}</p><button onClick={() => this.setState({ hasError: false, error: null })} style={{ padding: '0.5rem 1.5rem', fontSize: '1rem', cursor: 'pointer', borderRadius: '2px', border: '1px solid #1976d2', background: '#1976d2', color: '#fff' }}>重试</button></div>; return this.props.children; }
}

const NotFoundPage: React.FC = () => <div style={{ padding: '2rem', textAlign: 'center', marginTop: '10vh' }}><h1 style={{ fontSize: '4rem', color: '#ccc', margin: 0 }}>404</h1><p style={{ color: '#666', marginTop: '1rem' }}>页面未找到</p></div>;

const App: React.FC = () => (<ErrorBoundary><UserProvider><Routes><Route element={<Layout />}>
  {/* 一级: 两大入口卡片 */}
  <Route path="/" element={<HomePage />} />

  {/* 送样分支: /sample → portal → entry/list/stats */}
  <Route path="/sample" element={<SamplePortal />} />
  <Route path="/sample/:groupId" element={<SampleEntryPage />} />
  <Route path="/sample/stats" element={<SampleStatsPage />} />
  <Route path="/sample-records" element={<RdRecordsPage />} />

  {/* 工作量分支: /workload → portal → entry/stats/manage */}
  <Route path="/workload" element={<WorkloadPortal />} />
  <Route path="/entry/:groupId" element={<EntryPage />} />
  <Route path="/stats" element={<StatsPage />} />
  <Route path="/manage" element={<ManagePage />} />

  <Route path="/404" element={<NotFoundPage />} />
  <Route path="*" element={<Navigate to="/404" replace />} />
</Route></Routes></UserProvider></ErrorBoundary>);
export default App;
