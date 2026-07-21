import { atom, useAtom } from "jotai";
import { useEffect } from "react";

export interface AuthUser {
  userId: number;
  username: string;
  email: string;
  isSuperuser: boolean;
}

export type AuthState = "loading" | "authenticated" | "anonymous";

const tokenAtom = atom<string | null>(
  typeof window !== "undefined" ? localStorage.getItem("sso_token") : null
);

const authStateAtom = atom<AuthState>("loading");
const userAtom = atom<AuthUser | null>(null);

const setTokenAtom = atom(null, (_get, set, token: string | null) => {
  if (token) localStorage.setItem("sso_token", token);
  else localStorage.removeItem("sso_token");
  set(tokenAtom, token);
});

/** Verify stored token against backend on mount. */
const initAtom = atom(null, (get, set) => {
  const token = get(tokenAtom);
  if (!token) {
    set(authStateAtom, "anonymous");
    return;
  }
  set(authStateAtom, "loading");
  fetch("/api/creator/user/me", {
    headers: { Authorization: `Bearer ${token}` },
  })
    .then((r) => {
      if (!r.ok) throw new Error("unauthorized");
      return r.json();
    })
    .then((data) => {
      const u = data.user;
      set(userAtom, { userId: u.id, username: u.username, email: u.email, isSuperuser: u.is_superuser });
      set(authStateAtom, "authenticated");
    })
    .catch(() => {
      localStorage.removeItem("sso_token");
      set(tokenAtom, null);
      set(authStateAtom, "anonymous");
    });
});

// ── Hooks ──────────────────────────────────────────────

export function useAuth() {
  const [token, setToken] = useAtom(setTokenAtom);
  const [authState, setAuthState] = useAtom(authStateAtom);
  const [user, setUser] = useAtom(userAtom);
  const [, init] = useAtom(initAtom);

  // One-shot verification on mount
  useEffect(() => { init(); }, []);

  return {
    token,
    isAuthenticated: authState === "authenticated",
    isLoading: authState === "loading",
    user,
    login: (jwt: string) => {
      setToken(jwt);
      setAuthState("loading");
      // Verify with backend
      fetch("/api/creator/user/me", {
        headers: { Authorization: `Bearer ${jwt}` },
      })
        .then((r) => {
          if (!r.ok) throw new Error("unauthorized");
          return r.json();
        })
        .then((data) => {
          const u = data.user;
          setUser({ userId: u.id, username: u.username, email: u.email, isSuperuser: u.is_superuser });
          setAuthState("authenticated");
        })
        .catch(() => {
          setToken(null);
          setAuthState("anonymous");
        });
    },
    logout: () => {
      setToken(null);
      setUser(null);
      setAuthState("anonymous");
    },
  };
}
