import React, { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import type { User } from './types';

interface UserContextType {
  user: User | null;
  token: string | null;
  isLoggedIn: boolean;
  userName: string;
  setUserName: (name: string) => void;
  login: (user: User, token: string, remember?: boolean) => void;
  logout: () => void;
}

const UserContext = createContext<UserContextType>({
  user: null, token: null, isLoggedIn: false,
  userName: '', setUserName: () => {},
  login: () => {}, logout: () => {},
});

const TK = 'workload_token';
const UK = 'workload_user';
const RK = 'workload_remember';

const loadStored = (): { token: string | null; user: User | null } => {
  try {
    const storage = localStorage.getItem(RK) === 'true' ? localStorage : sessionStorage;
    const token = storage.getItem(TK);
    const userRaw = storage.getItem(UK);
    const user = userRaw ? JSON.parse(userRaw) : null;
    return { token, user };
  } catch {
    return { token: null, user: null };
  }
};

export const UserProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const stored = loadStored();
  const [token, setToken] = useState<string | null>(stored.token);
  const [user, setUser] = useState<User | null>(stored.user);
  const [userName, setUserNameState] = useState(stored.user?.username || '');

  const isLoggedIn = !!token && !!user;

  const login = useCallback((u: User, t: string, remember?: boolean) => {
    const storage = remember ? localStorage : sessionStorage;
    storage.setItem(TK, t);
    storage.setItem(UK, JSON.stringify(u));
    if (remember) {
      localStorage.setItem(RK, 'true');
    } else {
      localStorage.removeItem(RK);
    }
    setToken(t);
    setUser(u);
    setUserNameState(u.username || '');
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem(TK);
    localStorage.removeItem(UK);
    localStorage.removeItem(RK);
    sessionStorage.removeItem(TK);
    sessionStorage.removeItem(UK);
    setToken(null);
    setUser(null);
    setUserNameState('');
  }, []);

  const setUserName = useCallback((name: string) => {
    sessionStorage.setItem('workload_user_name', name);
    setUserNameState(name);
  }, []);

  return (
    <UserContext.Provider value={{ user, token, isLoggedIn, userName, setUserName, login, logout }}>
      {children}
    </UserContext.Provider>
  );
};

export const useUser = () => useContext(UserContext);
