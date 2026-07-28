// Approval Components · 审批工作区组件库

export { ApprovalCard } from "./ApprovalCard";
export { ApprovalPanel } from "./ApprovalPanel";

export { createApprovalPage } from "./createApprovalPage";
export type { ApprovalPageOptions } from "./createApprovalPage";

export { ApproverPicker } from "./ApproverPicker";
export type {
  ApproverPickerProps,
  ApproverPickerLabels,
  ApproverOption,
  ApproverRef,
} from "./ApproverPicker";

export { FlowGallery, GalleryMiniPreview } from "./FlowGallery";
export type {
  FlowGalleryProps,
  FlowGalleryLabels,
  FlowItem,
  FlowTemplate,
  GalleryMiniPreviewProps,
} from "./FlowGallery";


export { TimelineView } from "./TimelineView";
export type { TimelineNode } from "./TimelineView";
export { ApprovalNodeChain } from "./ApprovalNodeChain";
export type { ChainNode } from "./ApprovalNodeChain";

// Modal Components
export { ApprovalDetailModal } from "./ApprovalDetailModal";
export type { ApprovalDetailModalProps } from "./ApprovalDetailModal";
export { DelegationModal } from "./DelegationModal";
export type { DelegationModalProps, DelegationModalLabels } from "./DelegationModal";
export { WfNewRequestModal } from "./WfNewRequestModal";
export type { WfNewRequestModalProps } from "./WfNewRequestModal";

export type {
  ApprovalItem,
  ApprovalStatus,
  ApprovalTab,
  ApprovalTabId,
  ApprovalCardProps,
  ApprovalPanelProps,
} from "./types";
