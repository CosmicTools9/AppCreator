/**
 * FlowDesigner — shared types
 */
export interface FlowEdge { to: number; label?: string; cond?: string; }
export interface FlowNode {
  id: string; type: string; label: string; x?: number; y?: number;
  role?: string; roleKind?: 'role' | 'engineer' | string; sla?: number; mode?: string;
  expr?: string; recipients?: string; target?: string; branches?: number; outcome?: string;
  next?: FlowEdge[];
  [key: string]: unknown;
}
export type PortSide = 'top' | 'right' | 'bottom' | 'left';
export interface NodeTypeConfig { type: string; label: string; desc: string; color: string; }
export type ValidationError = { type: string; message: string; idx?: number };
export type ValidationResult = { valid: boolean; errors: ValidationError[] };
export interface FlowDesignerToolbarCtrl {
  undo: () => void; redo: () => void; canUndo: boolean; canRedo: boolean;
  save: () => void; togglePalette: () => void; paletteOpen: boolean;
  toggleDrawer: () => void; propOpen: boolean;
  validate: () => boolean; simulate: () => void;
  validation: ValidationResult; highlightedPath: number[]; draftDirty: boolean;
  zoom: number; relayout: () => void; zoomIn: () => void; zoomOut: () => void; fitToScreen: () => void;
  onOpenSubflow?: (targetCode: string) => void; subflowStack?: Array<{ name: string }>; onExitSubflow?: () => void;
}
export interface FlowDesignerProps {
  initialNodes?: FlowNode[]; flowName?: string; flowId?: number;
  onSave: (nodes: FlowNode[], name: string) => void | Promise<void>;
  renderToolbar?: (ctrl: FlowDesignerToolbarCtrl) => React.ReactNode;
  renderInspector?: (props: { node: FlowNode; onUpdate: (patch: Partial<FlowNode>) => void }) => React.ReactNode;
  onEnterSubflow?: (targetCode: string) => void; subflowStack?: Array<{ name: string }>; onExitSubflow?: () => void;
  nodeTypeLabels?: Partial<Record<string, { label: string; desc: string }>>; nodeTypes?: NodeTypeConfig[]; draftRestoredLabel?: string; discardDraftLabel?: string;
}
