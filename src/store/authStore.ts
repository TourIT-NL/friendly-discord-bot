import { create } from "zustand";

interface LogEntry {
  timestamp: string;
  level: "info" | "warn" | "error" | "debug" | "trace";
  message: string;
  metadata?: any;
}

interface AuthState {
  isAuthenticated: boolean;
  needsCredentials: boolean;
  user: {
    id: string;
    username: string;
    avatar?: string;
    email?: string;
  } | null;
  guilds: any[] | null;
  isLoading: boolean;
  error: string | null;
  lastErrorMetadata: any | null;
  retryCount: number;
  logs: LogEntry[];
  showDevLog: boolean;
  isLocked: boolean;
  hasMasterPassword: boolean;
  view:
    | "manual"
    | "auth"
    | "setup"
    | "qr"
    | "token"
    | "dashboard"
    | "unlock"
    | "set-master";
  setAuthenticated: (user: AuthState["user"]) => void;
  setUnauthenticated: () => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null, metadata?: any) => void;
  addLog: (level: LogEntry["level"], message: string, metadata?: any) => void;
  clearLogs: () => void;
  toggleDevLog: () => void;
  incrementRetry: () => void;
  resetRetry: () => void;
  setGuilds: (guilds: any[]) => void;
  setNeedsCredentials: (needs: boolean) => void;
  setView: (view: AuthState["view"]) => void;
  setLocked: (locked: boolean) => void;
  setHasMasterPassword: (has: boolean) => void;
  reset: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  isAuthenticated: false,
  needsCredentials: false,
  user: null,
  guilds: null,
  isLoading: false,
  error: null,
  lastErrorMetadata: null,
  retryCount: 0,
  logs: [],
  showDevLog: false,
  isLocked: false,
  hasMasterPassword: false,
  view: "manual",
  setAuthenticated: (user) =>
    set({
      isAuthenticated: true,
      user,
      isLoading: false,
      error: null,
      needsCredentials: false,
      retryCount: 0,
    }),
  setUnauthenticated: () =>
    set({
      isAuthenticated: false,
      user: null,
      guilds: null,
      isLoading: false,
      error: null,
      retryCount: 0,
    }),
  setLoading: (loading) => set({ isLoading: loading }),
  setError: (error, metadata = null) =>
    set({ error, lastErrorMetadata: metadata, isLoading: false }),
  addLog: (level, message, metadata = null) =>
    set((state) => ({
      logs: [
        {
          timestamp: new Date().toLocaleTimeString(),
          level,
          message,
          metadata,
        },
        ...state.logs,
      ].slice(0, 100),
    })),
  clearLogs: () => set({ logs: [] }),
  toggleDevLog: () => set((state) => ({ showDevLog: !state.showDevLog })),
  incrementRetry: () => set((state) => ({ retryCount: state.retryCount + 1 })),
  resetRetry: () => set({ retryCount: 0 }),
  setGuilds: (guilds) => set({ guilds, isLoading: false }),
  setNeedsCredentials: (needs) =>
    set({ needsCredentials: needs, isLoading: false }),
  setView: (view) => set({ view }),
  setLocked: (locked) => set({ isLocked: locked }),
  setHasMasterPassword: (has) => set({ hasMasterPassword: has }),
  reset: () =>
    set({
      isAuthenticated: false,
      needsCredentials: false,
      user: null,
      guilds: null,
      isLoading: false,
      error: null,
      lastErrorMetadata: null,
      retryCount: 0,
      logs: [],
      showDevLog: false,
      isLocked: false,
      hasMasterPassword: false,
      view: "manual",
    }),
}));
