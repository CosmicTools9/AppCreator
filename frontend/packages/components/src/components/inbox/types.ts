/**
 * 站内信组件类型定义
 *
 * 符合 Gateway 设计规范 §9.3 站内信工作区规范
 */

/** 消息类型 */
export type InboxMessageType = "system" | "message" | "approval" | "mention";

/** 站内信消息项 */
export interface InboxMessage {
  /** 唯一标识 */
  id: string | number;
  /** 发件人名称 */
  from: string;
  /** 头像 URL 或首字母 */
  avatar?: string;
  /** 消息标题 */
  title: string;
  /** 消息内容 */
  content: string;
  /** 时间描述 */
  time: string;
  /** 是否未读 */
  unread: boolean;
  /** 消息类型 */
  type: InboxMessageType;
}

/** 消息过滤 Tab */
export type InboxTabId = "all" | "unread" | "system";

/** Tab 配置项 */
export interface InboxTab {
  id: InboxTabId;
  label: string;
  count?: number;
}

/** 消息卡片 Props */
export interface InboxMessageCardProps {
  /** 消息数据 */
  message: InboxMessage;
  /** 是否被选中 */
  selected?: boolean;
  /** 点击卡片 */
  onClick?: (message: InboxMessage) => void;
  /** 删除消息 */
  onDelete?: (id: string | number) => void;
  /** 自定义类名 */
  className?: string;
}

/** 消息详情 Props */
export interface InboxMessageDetailProps {
  /** 当前消息 */
  message: InboxMessage;
  /** 返回列表 */
  onBack?: () => void;
  /** 删除消息 */
  onDelete?: (id: string | number) => void;
  /** 回复消息 */
  onReply?: (id: string | number, content: string) => void;
  /** 自定义类名 */
  className?: string;
}

/** 站内信面板 Props */
export interface InboxPanelProps {
  /** 消息列表数据 */
  messages: InboxMessage[];
  /** 当前激活的 Tab */
  activeTab?: InboxTabId;
  /** Tab 切换回调 */
  onTabChange?: (tab: InboxTabId) => void;
  /** 点击消息项回调（查看详情） */
  onMessageClick?: (message: InboxMessage) => void;
  /** 删除消息回调 */
  onDelete?: (id: string | number) => void;
  /** 全部已读回调 */
  onMarkAllRead?: () => void;
  /** 回复消息回调 */
  onReply?: (id: string | number, content: string) => void;
  /** 加载状态 */
  loading?: boolean;
  /** 自定义类名 */
  className?: string;
  // ── 发送站内信 ──
  /** 可选联系人列表（用于发送表单选择收件人/抄送人） */
  contacts?: import("../form/ContactMultiSelect").ContactOption[];
  /** 发送站内信回调 */
  onSend?: (params: InboxSendParams) => void;
  /** 发送表单加载状态 */
  sending?: boolean;
}

/** 触发器 Props */
export interface InboxTriggerProps {
  /** 未读数量徽标 */
  unreadCount?: number;
  /** 点击回调 */
  onClick?: () => void;
  /** 自定义类名 */
  className?: string;
}

/** 站内信发送参数 */
export interface InboxSendParams {
  /** 收件人 ID 列表 */
  to: string[];
  /** 抄送人 ID 列表 */
  cc: string[];
  /** 消息主题 */
  subject: string;
  /** 消息正文 */
  body: string;
}

/** 站内信发送表单 Props */
export interface InboxSendFormProps {
  /** 可选联系人列表（从 contacts 模块加载） */
  contacts: import("../form/ContactMultiSelect").ContactOption[];
  /** 返回列表视图 */
  onBack?: () => void;
  /** 发送消息回调 */
  onSend?: (params: InboxSendParams) => void;
  /** 加载状态 */
  loading?: boolean;
  /** 自定义类名 */
  className?: string;
}

/** 工作区 Props */
export interface InboxWorkspaceProps
  extends InboxPanelProps,
    Omit<InboxTriggerProps, "onClick"> {
  /** Sheet 打开状态（受控模式） */
  open?: boolean;
  /** Sheet 状态变化回调 */
  onOpenChange?: (open: boolean) => void;
}
