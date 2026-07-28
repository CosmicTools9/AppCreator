// Re-export all components from the shared library
// This file serves as the entry point for @alioth/components

export { cn } from "./lib/utils";

// Shadow DOM support
export {
  ShadowRootContext,
  useShadowRoot,
} from "./contexts/ShadowRootContext";

// Export styles
import "./styles.css";

// Export design tokens
export * as tokens from "./tokens";

// UI Components
export { Button, buttonVariants } from "./components/ui/button";
export { DynamicIcon } from "./components/ui/dynamic-icon";
export type { DynamicIconProps } from "./components/ui/dynamic-icon";
export {
  Card,
  CardHeader,
  CardFooter,
  CardTitle,
  CardDescription,
  CardAction,
  CardContent,
} from "./components/ui/card";
export { Input } from "./components/ui/input";
export { Label } from "./components/ui/label";
export { Checkbox } from "./components/ui/checkbox";
export {
  Toaster,
  toast,
  useToast,
  type ToastType,
} from "./components/ui/sonner";
export { Badge, badgeVariants } from "./components/ui/badge";
export { Avatar, AvatarImage, AvatarFallback } from "./components/ui/avatar";
export { Alert, AlertTitle, AlertDescription } from "./components/ui/alert";
export { Switch } from "./components/ui/switch";
export { Textarea } from "./components/ui/textarea";
export { Separator } from "./components/ui/separator";
export { Skeleton } from "./components/ui/skeleton";
export { Progress } from "./components/ui/progress";
export { ProgressBar, type ProgressBarProps } from "./components/ui/progress-bar";
export { RatingBadge, type RatingBadgeProps, type RatingLevel } from "./components/ui/rating-badge";
export { DeviceStatus, type DeviceStatusProps } from "./components/ui/device-status";
export { StatusBadge, type StatusBadgeProps } from "./components/ui/status-badge";
export {
  OrderCateBadge,
  ORDER_CATE_COLORS,
} from "./components/ui/badge-tf";
export { FilterPill, type FilterPillProps } from "./components/ui/filter-pill";
export { Timeline, type TimelineProps, type TimelineItem } from "./components/ui/timeline";
export { StatCard, StatGrid, type StatCardProps, type StatGridProps } from "./components/ui/stat-card";
export { FormRow, type FormRowProps } from "./components/ui/form-row";
export { Calendar } from "./components/ui/calendar";
export {
  Command,
  CommandDialog,
  CommandInput,
  CommandList,
  CommandEmpty,
  CommandGroup,
  CommandItem as CommandMenuItem,
  CommandShortcut,
  CommandSeparator,
} from "./components/ui/command";
export { SearchSelect, type SearchSelectOption } from "./components/ui/search-select";
export {
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
} from "./components/ui/tabs";
export {
  Popover,
  PopoverTrigger,
  PopoverContent,
} from "./components/ui/popover";
export {
  useFormField,
  Form,
  FormItem,
  FormLabel,
  FormControl,
  FormDescription,
  FormMessage,
  FormField,
} from "./components/ui/form";

// Theme Components
export {
  ThemeProvider,
  type ThemeProviderProps,
} from "./components/ui/theme-provider";
export { ThemeToggle } from "./components/ui/theme-toggle";

// Navigation Components
export {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbPage,
  BreadcrumbSeparator,
  BreadcrumbEllipsis,
} from "./components/ui/breadcrumb";
export { Stepper, type Step, type StepperProps } from "./components/ui/stepper";

export {
  Select,
  SelectGroup,
  SelectValue,
  SelectTrigger,
  SelectContent,
  SelectLabel,
  SelectItem,
  SelectSeparator,
} from "./components/ui/select";
export {
  Dialog,
  DialogPortal,
  DialogOverlay,
  DialogClose,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
} from "./components/ui/dialog";
export {
  Sheet,
  SheetPortal,
  SheetOverlay,
  SheetTrigger,
  SheetClose,
  SheetContent,
  SheetHeader,
  SheetFooter,
  SheetTitle,
  SheetDescription,
} from "./components/ui/sheet";
export {
  StandardDrawer,
  StandardDrawerPortal,
  StandardDrawerOverlay,
  StandardDrawerTrigger,
  StandardDrawerClose,
  StandardDrawerContent,
  StandardDrawerHeader,
  StandardDrawerFooter,
  StandardDrawerTitle,
  StandardDrawerDescription,
} from "./components/ui/standard-drawer";
export {
  AlertDialog,
  AlertDialogPortal,
  AlertDialogOverlay,
  AlertDialogTrigger,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogAction,
  AlertDialogCancel,
} from "./components/ui/alert-dialog";
export {
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableHead,
  TableRow,
  TableCell,
  TableCaption,
} from "./components/ui/table";

// Dropdown Menu Components
export {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuCheckboxItem,
  DropdownMenuRadioItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuGroup,
  DropdownMenuPortal,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuRadioGroup,
} from "./components/ui/dropdown-menu";

// Tooltip Components
export {
  Tooltip,
  TooltipTrigger,
  TooltipContent,
  TooltipProvider,
} from "./components/ui/tooltip";
export {
  HoverCard,
  HoverCardTrigger,
  HoverCardContent,
} from "./components/ui/hover-card";
export {
  NavigationMenu,
  NavigationMenuList,
  NavigationMenuItem,
  NavigationMenuContent,
  NavigationMenuTrigger,
  NavigationMenuLink,
  NavigationMenuIndicator,
  NavigationMenuViewport,
} from "./components/ui/navigation-menu";
export { ScrollArea, ScrollBar } from "./components/ui/scroll-area";
export { ResizablePanelGroup, ResizablePanel, ResizableHandle } from "./components/ui/resizable";
export { Toggle, toggleVariants } from "./components/ui/toggle";
export { ToggleGroup, ToggleGroupItem } from "./components/ui/toggle-group";
export { RadioGroup, RadioGroupItem } from "./components/ui/radio-group";
export { Pagination, type PaginationProps } from "./components/ui/pagination";


// Layout Components
// createModuleLayout and ModuleLayoutOptions moved to @alioth/composables
// ModuleLayout and types moved to @alioth/composables
export {
  ResponsiveGrid,
  type ResponsiveGridProps,
  type ResponsiveGridCols,
} from "./components/layout/responsive-grid";
export { MainNav } from "./components/layout/MainNav";
export type { MainNavItem, MainNavProps } from "./components/layout/MainNav";
export { TopBar } from "./components/layout/TopBar";
export { ScrollTabs } from "./components/layout/ScrollTabs";

// Approval Workflow
export { TimelineView } from "./components/approval/TimelineView";
export type { TimelineNode } from "./components/approval/TimelineView";
export { ApprovalNodeChain } from "./components/approval/ApprovalNodeChain";
export type { ChainNode } from "./components/approval/ApprovalNodeChain";
export type { ScrollTabsProps } from "./components/layout/ScrollTabs";
export { ContentArea } from "./components/layout/ContentArea";
export { Footer } from "./components/layout/Footer";
export {
  EmbeddedContext,
  useEmbedded,
} from "./components/layout/EmbeddedContext";
export {
  setModuleSidebar,
  clearModuleSidebar,
  useModuleSidebar,
  type ModuleSidebarBranding,
} from "./components/layout/module-sidebar-atoms";
export { DetailPanel, type DetailPanelProps, type DetailMetaItem } from "./components/layout/DetailPanel";
export type {
  TopBarProps,
  BreadcrumbItemType,
} from "./components/layout/TopBar";
export type { ContentAreaProps } from "./components/layout/ContentArea";
export type { FooterProps, FooterLink } from "./components/layout/Footer";

// Feedback Components
export {
  ErrorBoundary,
  ErrorFallback,
  withErrorBoundary,
  type ErrorFallbackProps,
  type ErrorBoundaryProps,
} from "./components/feedback/error-boundary";
export {
  LoadingOverlay,
  type LoadingOverlayProps,
} from "./components/feedback/loading-overlay";
export {
  EmptyState,
  type EmptyStateProps,
} from "./components/feedback/empty-state";

// Hooks
export {
  useNotification,
  type UseNotificationReturn,
  type NotificationOptions,
  type ErrorNotificationOptions,
  type PromiseMessages,
} from "./hooks/use-notification";
export {
  useFormErrorReporter,
  type UseFormErrorReporterReturn,
} from "./hooks/use-form-error-reporter";
export {
  useEphemeralState,
  useEphemeralStateSync,
  type UseEphemeralStateReturn,
  type EphemeralValue,
  type UseEphemeralStateSyncOptions,
} from "./hooks/use-ephemeral-state";
export {
  useCommandPalette,
  type CommandItem,
  type GroupedItems,
} from "./hooks/useCommandPalette";
// Layout Components - Command Palette



// Form Components
// AutoForm and types moved to @alioth/composables
export {
  SearchableSelect,
  type SearchableSelectOption,
  type SearchableSelectProps,
} from "./components/form/SearchableSelect";
// ReferenceSelect and ReferenceSelectProps moved to @alioth/composables
export {
  ContactMultiSelect,
  type ContactOption,
  type ContactMultiSelectProps,
} from "./components/form/ContactMultiSelect";
export {
  MultiSelect,
  type MultiSelectOption,
  type MultiSelectProps,
} from "./components/form/MultiSelect";
export {
  TagInput,
  type TagInputProps,
} from "./components/form/TagInput";
export {
  FileUpload,
  type FileUploadProps,
} from "./components/form/FileUpload";
export {
  ScalarField,
  type ScalarFieldProps,
} from "./components/form/ScalarField";
export {
  DateTimePicker,
  type DateTimePickerProps,
} from "./components/form/DateTimePicker";
export {
  CascadingSelect,
  type CascadingOption,
  type CascadingSelectProps,
} from "./components/form/CascadingSelect";
export {
  DateRangePicker,
  type DateRangeValue,
  type DateRangePickerProps,
} from "./components/form/DateRangePicker";
export {
  RichTextEditor,
  type RichTextEditorProps,
} from "./components/form/RichTextEditor";
export type { ColumnDef } from "@tanstack/react-table";
export {
  DataTablePagination,
  type DataTablePaginationProps,
} from "./components/data/DataTablePagination";
export {
  InlineEditTable,
  type InlineEditTableProps,
  type InlineEditColumnDef,
} from "./components/data/InlineEditTable";

// Entity List (CRUD factory stubs)
export { createEntityListPage, createEntityTabbedListPage } from "./components/data/entity-list";
export type {
  EntityListPageConfig,
  EntityListPageHooks,
  EntityListTabConfig,
  EntityTabbedListPageConfig,
  InlineEditingConfig,
  TabbedInlineEditingConfig,
  ConfigurableEntityListPage,
  ConfigurableEntityTabbedListPage,
} from "./components/data/entity-list";



// AI Components (已迁移到独立 Modules/ai/，Framework 保留向后兼容)
export {
  AIChatPanel,
  type AIChatPanelProps,
  type AIMessage,
  type AgentOption,
} from "./components/ai/AIChatPanel";
export {
  AIWorkspace,
  type AIWorkspaceProps,
} from "./components/ai/AIWorkspace";
// AI Context — 页面上下文注册机制
export {
  aiContextAtom,
  useProvideAIContext,
  useAIContext,
} from "./components/ai/ai-context";
export type { AIPageContext, AIContextState } from "./components/ai/page-context";
// PageContextModule — 深度页面上下文模块
export {
  pageContextModule,
  PageContextModule,
} from "./components/ai/page-context";
export type {
  ContextEnvelope,
  RenderedContext,
} from "./components/ai/page-context";


// Workspace Atoms
export {
  activeWorkspaceAtom,
  openWorkspaceAtom,
  closeWorkspaceAtom,
  toggleWorkspaceAtom,
} from "./components/workspace/workspace-atoms";
export type { WorkspaceId } from "./components/workspace/workspace-atoms";
export type { WorkspaceSlot } from "./components/workspace";

// Workspace Components
export { WorkspaceDock, WorkspaceHub, WorkspaceShell, WorkspaceTrigger } from "./components/workspace";
export type { WorkspaceDockProps, WorkspaceHubProps, WorkspaceTriggerProps } from "./components/workspace";

// Approval Panel Components
export {
  ApprovalCard,
  ApprovalPanel,
  createApprovalPage,
  ApproverPicker,
  FlowGallery,
  GalleryMiniPreview,
  ApprovalDetailModal,
  DelegationModal,
  WfNewRequestModal,
} from "./components/approval";
export type {
  ApprovalItem,
  ApprovalStatus,
  ApprovalTab,
  ApprovalTabId,
  ApprovalCardProps,
  ApprovalPanelProps,
  ApproverPickerProps,
  ApproverPickerLabels,
  ApproverOption,
  ApproverRef,
  FlowGalleryProps,
  FlowGalleryLabels,
  FlowItem,
  FlowTemplate,
  GalleryMiniPreviewProps,
  ApprovalDetailModalProps,
  DelegationModalProps,
  DelegationModalLabels,
  WfNewRequestModalProps,
} from "./components/approval";

// Inbox Panel Components
export { InboxMessageCard, InboxMessageDetail, InboxPanel, InboxSendForm } from "./components/inbox";
export type {
  InboxMessage,
  InboxMessageType,
  InboxTabId,
  InboxTab,
  InboxMessageCardProps,
  InboxMessageDetailProps,
  InboxPanelProps,
  InboxSendParams,
  InboxSendFormProps,
} from "./components/inbox";

// Schedule Panel Components
export { MiniCalendar, WeekView, ScheduleTimeline, TodoList, QuickAddForm, SchedulePanel, GanttChart, createSchedulePage } from "./components/schedule";
export type {
  SchedulePlanType,
  MiniCalendarProps,
  WeekViewProps,
  ScheduleTimelineProps,
  TodoListProps,
  QuickAddFormProps,
  SchedulePanelProps,
  GanttChartProps,
  GanttItem,
  GanttData,
  SchedulePlan,
  ScheduleEvent,
  ScheduleItem,
  TodoItem,
  TodoObject,
  ScheduleViewMode,
  ScheduleTab,
  ReminderSetting,
  ReminderOffset,
  Participant,
  DateTimeSpan,
  LinkedApproval,
} from "./components/schedule";

// System Config Panel Components
export { SystemConfigPanel } from "./components/system-config";
export type {
  ConfigCategory,
  ConfigCategoryCode,
  SystemConfig,
  CreateSystemConfigRequest,
  UpdateSystemConfigRequest,
  SystemConfigPanelProps,
} from "./components/system-config";

// Dashboard Components
export { StatCard as DashboardStatCard, QuickLink, ActivityItem } from "./components/dashboard";
export type { StatCardProps as DashboardStatCardProps, QuickLinkProps, ActivityItemProps } from "./components/dashboard";






// Flow Designer Components
export { FlowDesigner } from './components/flow/FlowDesigner';
export { FlowNodePalette } from './components/flow/FlowDesignerToolbar';
export { FlowToolbar } from './components/flow/FlowToolbar';
export { evaluateExpr } from './components/flow/expression';
export { simulateFlow } from './components/flow/simulation';
export { validateFlow } from './components/flow/validation';
export { serializeFlow, deserializeFlow } from './components/flow/flow-persistence';
export type { FlowMeta, FlowGraphPayload } from './components/flow/flow-persistence';
export { approvalTabAtom, selectedApprovalIdAtom, workflowDesignerScreenAtom, newRequestModalAtom, delegationModalAtom, pendingFilterAtom } from './components/flow/flow-atoms';
export type { TabKey, Screen } from './components/flow/flow-atoms';
export { effectiveNext, getNodeSize, ensurePositions, autoLayout, nodeColor, nodeIcon, NODE_TYPES, NODE_W, NODE_H, PAD, COLS, X_GAP, Y_GAP } from './components/flow/utils';
export { FlowInspector } from "./components/flow/FlowInspector";
export type { FlowInspectorProps, FlowInspectorLabels } from "./components/flow/FlowInspector";
export type { FlowNode, FlowEdge, PortSide, NodeTypeConfig, FlowDesignerProps, FlowDesignerToolbarCtrl, ValidationResult, ValidationError } from './components/flow/types';

// I18n dictionaries
export { default as componentsZhCN } from "./locales/zh-CN.json";
export { default as componentsEn } from "./locales/en.json";
