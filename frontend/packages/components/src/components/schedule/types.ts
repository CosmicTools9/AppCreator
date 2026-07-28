/**
 * Schedule Components · 日程管理组件类型定义
 *
 * 基于 zc_id_plan（计划）+ zc_id_event（事件）双表模型设计。
 * 日程项由 Plan 为主体，通过 zc_id_plan_rr_event 关联 Event 补充执行侧信息。
 *
 * 数据组装路径：
 *   Plan（标题/类型/时间/周期/进度）
 *     → zc_id_plan_rr_event
 *     → Event（地点/参与人/审批状态）
 */

// ============================================
// Core Domain Types
// ============================================

/** 提醒提前时间（分钟） */
export type ReminderOffset = 0 | 5 | 15 | 30 | 60 | 1440;

/** 提醒设置 */
export interface ReminderSetting {
 /** 提前提醒时间（分钟），0 表示不提醒 */
 offset: ReminderOffset;
 /** 提醒方式 */
 channel?: "app" | "email" | "sms";
}

/** 日程计划类型（映射 zc_id_plan._t_） */
export type SchedulePlanType =
 | "meeting"
 | "sync"
 | "client"
 | "development"
 | "dev"
 | "team"
 | "review"
 | "personal"
 | "other";

/** 审批状态（映射 zc_id_even-approve.kanban_status） */
export type ApprovalStatus = "pending" | "approved" | "rejected";

/** 关联审批信息（审批联动） */
export interface LinkedApproval {
 /** 审批单据 ID（event id） */
 id: string | number;
 /** 审批标题 */
 title: string;
 /** 审批状态 */
 status: ApprovalStatus;
 /** 申请人 */
 applicant?: string;
}

/** 参与主体信息 */
export interface Participant {
 /** 主体 ID */
 id: string | number;
 /** 主体名称 */
 name: string;
 /** 参与角色 */
 role?: string;
}

// ============================================
// Plan & Event Primitives
// ============================================

/** 日程计划（对应 zc_id_plan） */
export interface SchedulePlan {
 /** 计划 ID（zuid） */
 id: string | number;
 /** 标题（notice） */
 title: string;
 /** 业务类型（_t_） */
 type: SchedulePlanType;
 /** 日期（YYYY-MM-DD，由 qk_date-segm 解析） */
 date: string;
 /** 时间（HH:MM，由 qk_time-segm 解析） */
 time: string;
 /** 时长描述 */
 duration: string;
 /** 周期性规则（cron） */
 cron?: string;
 /** 排除日期列表 */
 excludeDates?: string[];
 /** 完成进度（通过 zc_id_rati-progress 标量引用解析，progress_pct） */
 progressPct: number;
 /** 排期进度百分比（schedule_pct） */
 schedulePct: number;
 /** 排序权重 */
 sort: number;
 /** 创建时间 */
 createdAt: string;
 /** 更新时间 */
 updatedAt: string;
}

/** 日程事件（对应 zc_id_event，通过 zc_id_plan_rr_event 关联） */
export interface ScheduleEvent {
 /** 事件 ID（zuid） */
 id: string | number;
 /** 关联计划 ID */
 planId: string | number;
 /** 事件标题（notice） */
 title?: string;
 /** 地点（fk_place → zc_id_place.notice） */
 location?: string;
 /** 参与主体（fk_subject → zc_id_subjects.notice） */
 subject?: string;
 /** 关联审批 */
 linkedApproval?: LinkedApproval;
 /** 创建时间 */
 createdAt: string;
}

/** 日期/时间跨度 */
export interface DateTimeSpan {
 /** 日期开始 YYYY-MM-DD */
 dateStart?: string;
 /** 日期结束 YYYY-MM-DD */
 dateEnd?: string;
 /** 时间开始 HH:MM */
 timeStart?: string;
 /** 时间结束 HH:MM */
 timeEnd?: string;
}

/** 组装后的日程展示项（Plan + 关联 Event 信息 + segm-date 跨度） */
export interface ScheduleItem {
 /** 计划 ID */
 id: string | number;
 /** 标题 */
 title: string;
 /** 类型 */
 type: SchedulePlanType;
 /** 日期/时间跨度（来自 zc_id_segm-date） */
 span: DateTimeSpan;
 /** 时长描述 */
 duration: string;
 /** 地点（来自关联 Event） */
 location?: string;
 /** 参与主体（来自关联 Event） */
 subject?: string;
 /** 参与人列表（来自 zc_id_plan_rr_participants） */
 participants?: Participant[];
 /** 是否已完成（progress_pct === 100） */
 done: boolean;
 /** 完成进度 */
 progressPct: number;
 /** 提醒设置（前端概念，存于 plan 扩展字段或独立表） */
 reminder?: ReminderSetting;
 /** 关联审批（来自关联 Event → even-approve） */
 linkedApproval?: LinkedApproval;
 /** 周期性规则 */
 cron?: string;
}

/** 待办客体（操作对象） */
export interface TodoObject {
 /** 客体 ID */
 id: string | number;
 /** 客体名称 */
 name: string;
 /** 客体类型：production（产品服务）| bill（单据）| other */
 type?: "production" | "bill" | "other";
}

/** 待办事项（贴合 zc_id_event 模型）
 *
 * 客体（objects）代表真正需要做的事，如操作产品服务、操作单据等。
 * 完成状态由关联的 zc_id_stus-event 状态推导。
 */
export interface TodoItem {
 /** 事件 ID */
 id: string | number;
 /** 标题（来自 notice） */
 title: string;
 /** 执行主体 */
 subject?: string;
 /** 客体列表（真正需要做的事） */
 objects: TodoObject[];
 /** 截止时间（qk_date 解析） */
 dueDate?: string;
 /** 状态名称 */
 status?: string;
 /** 是否已完成 */
 done: boolean;
}

// ============================================
// View & Tab Types
// ============================================

/** 视图模式 */
export type ScheduleViewMode = "month" | "week" | "list";

/** Tab 配置项 */
export interface ScheduleTab {
 id: ScheduleViewMode;
 label: string;
}

// ============================================
// Component Props
// ============================================

/** 迷你月历 Props */
export interface MiniCalendarProps {
 /** 选中的日期 */
 selectedDate?: Date;
 /** 日期选择回调 */
 onSelectDate?: (date: Date) => void;
 /** 有日程的日期集合 (YYYY-MM-DD) */
 eventDates?: string[];
 /** 自定义类名 */
 className?: string;
}

/** 周视图 Props */
export interface WeekViewProps {
 /** 当前周基准日期 */
 baseDate?: Date;
 /** 该周的日程项 */
 items: ScheduleItem[];
 /** 日期选择回调 */
 onSelectDate?: (date: Date) => void;
 /** 点击日程项回调 */
 onItemClick?: (item: ScheduleItem) => void;
 /** 自定义类名 */
 className?: string;
}

/** 时间轴 Props */
export interface ScheduleTimelineProps {
 /** 日程项列表 */
 items: ScheduleItem[];
 /** 点击日程项回调 */
 onItemClick?: (item: ScheduleItem) => void;
 /** 点击关联审批回调 */
 onApprovalClick?: (approval: LinkedApproval) => void;
 /** 自定义类名 */
 className?: string;
}

/** 待办清单 Props */
export interface TodoListProps {
 /** 待办事项列表 */
 items: TodoItem[];
 /** 勾选状态变化回调 */
 onToggle?: (id: string | number) => void;
 /** 自定义类名 */
 className?: string;
}

/** 快速新建表单 Props */
export interface QuickAddFormProps {
 /** 添加日程项回调 */
 onAdd?: (item: Omit<ScheduleItem, "id" | "done" | "progressPct">) => void;
 /** 默认日期 */
 defaultDate?: string;
 /** 自定义类名 */
 className?: string;
}

/** 日程面板 Props */
export interface SchedulePanelProps {
 /** 日程项列表（已组装的 Plan + Event） */
 items: ScheduleItem[];
 /** 待办事项列表 */
 todos: TodoItem[];
 /** 当前视图模式 */
 viewMode?: ScheduleViewMode;
 /** 视图切换回调 */
 onViewModeChange?: (mode: ScheduleViewMode) => void;
 /** 日期选择回调 */
 onSelectDate?: (date: Date) => void;
 /** 添加日程项回调 */
 onAddItem?: (item: Omit<ScheduleItem, "id" | "done" | "progressPct">) => void;
 /** 切换待办状态回调 */
 onToggleTodo?: (id: string | number) => void;
 /** 点击日程项回调 */
 onItemClick?: (item: ScheduleItem) => void;
 /** 点击关联审批回调 */
 onApprovalClick?: (approval: LinkedApproval) => void;
 /** 加载状态 */
 loading?: boolean;
 /** 选中的日期 */
 selectedDate?: Date;
 /** 自定义类名 */
 className?: string;
}

/** 日程触发器 Props */
export interface ScheduleTriggerProps {
 /** 今日日程数量徽标 */
 eventCount?: number;
 /** 点击回调 */
 onClick?: () => void;
 /** 是否激活状态（面板打开） */
 active?: boolean;
 /** 自定义类名 */
 className?: string;
}

/** 日程工作区 Props（组合触发器 + 面板） */
export interface ScheduleWorkspaceProps extends SchedulePanelProps {
 /** 待处理日程数量（徽标用） */
 eventCount?: number;
 /** Sheet 打开状态（受控模式） */
 open?: boolean;
 /** Sheet 状态变化回调 */
 onOpenChange?: (open: boolean) => void;
}

// ============================================
// Gantt Chart Types
// ============================================
/** 甘特图条目 */
export interface GanttItem {
 id: string | number;
 name: string;
 plan_type: string;
 start_date: string;
 end_date: string;
 progress_pct: number | null;
 color: string;
 fk_place?: (string | number) | null;
 fk_place_name?: string | null;
 fk_subject?: (string | number) | null;
 fk_subject_name?: string | null;
}
/** 甘特图数据集 */
export interface GanttData {
 items: GanttItem[];
 range_start: string;
 range_end: string;
}
/** 甘特图组件 Props */
export interface GanttChartProps {
 data: GanttData;
}
