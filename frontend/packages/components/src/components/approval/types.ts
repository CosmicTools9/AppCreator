/**
 * 审批组件类型定义
 *
 * 符合 Gateway 设计规范 §9.2 审批单据卡片规范
 */

/** 审批状态 */
export type ApprovalStatus = 'pending' | 'approved' | 'rejected';

/** 审批单据项 */
export interface ApprovalItem {
  /** 唯一标识 */
  id: string | number;
  /** 审批标题 */
  title: string;
  /** 申请人 */
  applicant: string;
  /** 部门 */
  dept?: string;
  /** 审批编号 */
  code?: string;
  /** 审批类型 */
  type?: string;
  /** 审批状态 */
  status: ApprovalStatus;
  /** 时间描述 */
  time: string;
  /** 头像或首字母 */
  avatar?: string;
}

/** 审批 Tab 分类 */
export type ApprovalTabId = 'pending' | 'approved' | 'rejected' | 'my';

/** Tab 配置项 */
export interface ApprovalTab {
  id: ApprovalTabId;
  label: string;
  count?: number;
}

/** 审批面板 Props */
export interface ApprovalPanelProps {
  /** 审批列表数据 */
  items: ApprovalItem[];
  /** 当前激活的 Tab */
  activeTab?: ApprovalTabId;
  /** Tab 切换回调 */
  onTabChange?: (tab: ApprovalTabId) => void;
  /** 通过审批回调 */
  onApprove?: (id: string | number) => void;
  /** 驳回审批回调 */
  onReject?: (id: string | number) => void;
  /** 点击审批项回调（查看详情） */
  onItemClick?: (item: ApprovalItem) => void;
  /** 加载状态 */
  loading?: boolean;
  /** 自定义类名 */
  className?: string;
}

/** 审批卡片 Props */
export interface ApprovalCardProps {
  /** 审批数据 */
  item: ApprovalItem;
  /** 通过审批回调 */
  onApprove?: (id: string | number) => void;
  /** 驳回审批回调 */
  onReject?: (id: string | number) => void;
  /** 点击查看详情 */
  onClick?: (item: ApprovalItem) => void;
  /** 自定义类名 */
  className?: string;
}

/** 审批触发器 Props */
export interface ApprovalTriggerProps {
  /** 待审批数量徽标 */
  pendingCount?: number;
  /** 点击回调 */
  onClick?: () => void;
  /** 自定义类名 */
  className?: string;
}

/** 审批工作区 Props（组合触发器 + 面板） */
export interface ApprovalWorkspaceProps
  extends ApprovalPanelProps, Omit<ApprovalTriggerProps, 'onClick'> {
  /** Sheet 打开状态（受控模式） */
  open?: boolean;
  /** Sheet 状态变化回调 */
  onOpenChange?: (open: boolean) => void;
}
