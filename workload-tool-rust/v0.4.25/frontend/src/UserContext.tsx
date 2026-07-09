import React, { createContext, useContext, useState, type ReactNode } from 'react';

interface UserContextType {
  userName: string;
  setUserName: (name: string) => void;
}

const UserContext = createContext<UserContextType>({ userName: '', setUserName: () => {} });

const UK = 'workload_user_name';

export const UserProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const [userName, setUserNameState] = useState(() => sessionStorage.getItem(UK) || '');
  const setUserName = (name: string) => { sessionStorage.setItem(UK, name); setUserNameState(name); };
  return <UserContext.Provider value={{ userName, setUserName }}>{children}</UserContext.Provider>;
};

export const useUser = () => useContext(UserContext);
