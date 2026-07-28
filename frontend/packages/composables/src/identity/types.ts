/**
 * Identity Service — shared DTO types.
 *
 * Framework provides these types. Each namespace's Module frontend
 * imports them from `@alioth/composables/identity`.
 */

/** 工程师 */
export interface Engineer {
  id: number;
  name: string;
  code: string | null;
  fk_user: number | null;
  sk_currency: number | null;
  ck_category: number | null;
  sk_unit: number | null;
}

/** 技能标签 */
export interface SkillTag {
  id: number;
  name: string;
  code: string | null;
  v_group: string | null;
}

/** 审批角色 */
export interface ApprovalRole {
  id: number;
  name: string;
}

/** CCB 成员 */
export interface CCBMember {
  id: number;
  name: string;
  ck_category: number | null;
  description: string | null;
  role: string;
  weight: number;
}
