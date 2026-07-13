import React, { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { login as apiLogin, fetchMe, changePassword as apiChangePassword, TOKEN_KEY } from '../api/client';
import type { LoginData, MeData } from '../types/lims';

interface AuthContextType {
  token: string | null;
  user: MeData | null;
  permissions: string[];
  userName: string;
  /** 兼容旧页面：仅更新展示名（不触发登录） */
  setUserName: (name: string) => void;
  login: (username: string, password: string) => Promise<LoginData>;
  logout: () => void;
  changePassword: (oldPassword: string | undefined, newPassword: string) => Promise<void>;
  /** 权限判定：含 `*` 通配符放行 */
  hasPerm: (perm: string) => boolean;
  loading: boolean;
}

const AuthContext = createContext<AuthContextType>({
  token: null,
  user: null,
  permissions: [],
  userName: '',
  setUserName: () => {},
  login: async () => { throw new Error('未初始化'); },
  logout: () => {},
  changePassword: async () => {},
  hasPerm: () => false,
  loading: true,
});

export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [token, setToken] = useState<string | null>(() => localStorage.getItem(TOKEN_KEY));
  const [user, setUser] = useState<MeData | null>(null);
  const [loading, setLoading] = useState<boolean>(true);

  const permissions = user?.permissions ?? [];

  const loadMe = async (tk: string) => {
    try {
      const res = await fetchMe();
      if (res.code === 0 && res.data) {
        setUser(res.data);
        return;
      }
    } catch {
      // 令牌失效
    }
    localStorage.removeItem(TOKEN_KEY);
    setToken(null);
    setUser(null);
  };

  useEffect(() => {
    if (token) {
      loadMe(token).finally(() => setLoading(false));
    } else {
      setLoading(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const login = async (username: string, password: string): Promise<LoginData> => {
    const res = await apiLogin({ username, password });
    if (res.code !== 0 || !res.data) {
      throw new Error(res.message || '登录失败');
    }
    const data = res.data;
    localStorage.setItem(TOKEN_KEY, data.token);
    setToken(data.token);
    // 拉取最新档案（含权限）
    try {
      const me = await fetchMe();
      if (me.code === 0 && me.data) setUser(me.data);
    } catch {
      // 忽略；下次进入受保护路由会刷新
    }
    return data;
  };

  const logout = () => {
    localStorage.removeItem(TOKEN_KEY);
    setToken(null);
    setUser(null);
    window.location.href = '/login';
  };

  const changePassword = async (oldPassword: string | undefined, newPassword: string) => {
    const res = await apiChangePassword({ old_password: oldPassword, new_password: newPassword });
    if (res.code !== 0) throw new Error(res.message || '修改失败');
    // 刷新档案（must_change_password 已清除）
    if (token) {
      const me = await fetchMe();
      if (me.code === 0 && me.data) setUser(me.data);
    }
  };

  const hasPerm = (perm: string): boolean => {
    if (permissions.includes('*')) return true;
    return permissions.includes(perm);
  };

  const setUserName = (name: string) => {
    setUser((prev) => (prev ? { ...prev, username: name, display_name: name } : prev));
  };

  return (
    <AuthContext.Provider value={{ token, user, permissions, userName: user?.username ?? '', setUserName, login, logout, changePassword, hasPerm, loading }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => useContext(AuthContext);
