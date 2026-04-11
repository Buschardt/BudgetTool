import {
  createContext,
  useContext,
  useState,
  useCallback,
  type ReactNode,
} from 'react';
import * as api from './api';

interface AuthState {
  token: string | null;
  username: string | null;
}

interface AuthContextValue extends AuthState {
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

function decodeUsername(token: string): string | null {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    return typeof payload.username === 'string' ? payload.username : null;
  } catch {
    return null;
  }
}

function loadFromStorage(): AuthState {
  const token = localStorage.getItem('token');
  if (!token) return { token: null, username: null };
  return { token, username: decodeUsername(token) };
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>(loadFromStorage);

  const login = useCallback(async (username: string, password: string) => {
    const { token } = await api.login(username, password);
    const decodedUsername = decodeUsername(token) ?? username;
    localStorage.setItem('token', token);
    setState({ token, username: decodedUsername });
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem('token');
    setState({ token: null, username: null });
  }, []);

  return (
    <AuthContext.Provider value={{ ...state, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used inside AuthProvider');
  return ctx;
}
