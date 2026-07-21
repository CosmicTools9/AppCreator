import { atom, useAtom } from "jotai";

export interface AuthUser {
  userId: number;
  username: string;
  email: string;
  isSuperuser: boolean;
}

/** JWT token from SSO login  (null = not authenticated). */
const tokenAtom = atom<string | null>(
  typeof window !== "undefined" ? localStorage.getItem("sso_token") : null
);

/** Derived: authenticated state. */
const isAuthenticatedAtom = atom((get) => get(tokenAtom) !== null);

/** Current user info — filled after token verification. */
const userAtom = atom<AuthUser | null>(null);

/** Set token + persist to localStorage. */
const setTokenAtom = atom(null, (_get, set, token: string | null) => {
  if (token) {
    localStorage.setItem("sso_token", token);
  } else {
    localStorage.removeItem("sso_token");
  }
  set(tokenAtom, token);
});

// ── Hooks ──────────────────────────────────────────────

export function useAuth() {
  const [token, setToken] = useAtom(setTokenAtom);
  const [isAuth] = useAtom(isAuthenticatedAtom);
  const [user, setUser] = useAtom(userAtom);

  return {
    token,
    isAuthenticated: isAuth,
    user,
    login: (jwt: string, userInfo: AuthUser) => {
      setToken(jwt);
      setUser(userInfo);
    },
    logout: () => {
      setToken(null);
      setUser(null);
    },
  };
}
