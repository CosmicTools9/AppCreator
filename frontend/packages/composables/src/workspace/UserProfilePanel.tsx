/**
 * UserProfilePanel · 用户档案面板
 *
 * 自给自足的数据获取模式，直接对接后端 API：
 * - 用户信息 -> GET /api/auth/me
 *
 * 调用方仅需渲染该组件，无需传递数据。
 */
import * as React from "react";
import { useQuery } from "@tanstack/react-query";
import { apiClient } from "@alioth/api";
import { useT } from "@alioth/i18n";
import { Mail, Phone, Building, Briefcase, User, Loader2 } from "lucide-react";

// ============================================
// Types
// ============================================

interface UserProfile {
  id: number;
  name: string;
  email: string | null;
  phone: string | null;
  department: string | null;
  position: string | null;
  avatar_url: string | null;
  role: string | null;
}

interface AuthMeResponse {
  success: boolean;
  data?: UserProfile;
}

async function fetchUserProfile(): Promise<UserProfile | null> {
  try {
    const res = await apiClient.get<AuthMeResponse>("/auth/me");
    return res?.data ?? null;
  } catch {
    return null;
  }
}

// ============================================
// Default Profile Data (API 不可用时 fallback)
// ============================================

const DEMO_PROFILE: UserProfile = {
  id: 0,
  name: "Demo User",
  email: "demo@aliothstudio.com",
  phone: "+86-138-0000-0000",
  department: "IT Department",
  position: "Developer",
  avatar_url: null,
  role: "Admin",
};

// ============================================
// Component
// ============================================

export function UserProfilePanel(): React.ReactElement {
  const t = useT();
  const { data: profile, isLoading } = useQuery({
    queryKey: ["auth", "me"],
    queryFn: fetchUserProfile,
    staleTime: 5 * 60 * 1000,
  });

  const user = profile ?? (isLoading ? null : DEMO_PROFILE);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-48">
        <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!user) {
    return (
      <div className="p-6 text-center text-sm text-muted-foreground">
        <User className="w-10 h-10 mx-auto mb-2 opacity-40" />
        <p>{t("moduleLayout.profileUnavailable")}</p>
      </div>
    );
  }

  const initial = user.name.charAt(0).toUpperCase();

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      {/* 用户头部 */}
      <div className="flex flex-col items-center py-8 px-6 border-b">
        {user.avatar_url ? (
          <img
            src={user.avatar_url}
            alt={user.name}
            className="w-20 h-20 rounded-full object-cover mb-3"
          />
        ) : (
          <div className="w-20 h-20 rounded-full bg-primary/10 flex items-center justify-center mb-3">
            <span className="text-2xl font-bold text-primary">{initial}</span>
          </div>
        )}
        <p className="text-lg font-semibold">{user.name}</p>
        {user.role && (
          <span className="text-xs text-muted-foreground mt-0.5 px-2 py-0.5 rounded-full bg-muted">
            {user.role}
          </span>
        )}
      </div>

      {/* 详细信息 */}
      <div className="flex-1 p-4 space-y-3">
        <InfoRow icon={<Mail className="w-4 h-4" />} label={t("common.email")} value={user.email} />
        <InfoRow icon={<Phone className="w-4 h-4" />} label={t("common.phone")} value={user.phone} />
        <InfoRow icon={<Building className="w-4 h-4" />} label={t("common.department")} value={user.department} />
        <InfoRow icon={<Briefcase className="w-4 h-4" />} label={t("common.position")} value={user.position} />
      </div>
    </div>
  );
}

function InfoRow({ icon, label, value }: { icon: React.ReactNode; label: string; value: string | null }): React.ReactElement {
  return (
    <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-muted/30">
      <span className="text-muted-foreground shrink-0">{icon}</span>
      <div className="min-w-0 flex-1">
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="text-sm font-medium truncate">{value ?? "—"}</p>
      </div>
    </div>
  );
}
