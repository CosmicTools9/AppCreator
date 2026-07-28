import * as React from 'react';
import { Save, Undo2, Redo2 } from 'lucide-react';
export interface FlowToolbarProps {
  onUndo: () => void; onRedo: () => void; canUndo: boolean; canRedo: boolean; onSave: () => void;
  undoTitle?: string; redoTitle?: string; saveLabel?: string;
}
export function FlowToolbar({ onUndo, onRedo, canUndo, canRedo, onSave, undoTitle = 'Undo', redoTitle = 'Redo', saveLabel = 'Save' }: FlowToolbarProps): React.ReactElement {
  return (
    <div className="vfd-topbar" style={{ padding: '8px 16px', borderBottom: '1px solid hsl(var(--border))', background: 'hsl(var(--card))' }}>
      <button type="button" onClick={onUndo} disabled={!canUndo} className="btn btn-ghost btn-sm" style={{ opacity: canUndo ? 1 : 0.3 }} title={undoTitle}><Undo2 className="w-3.5 h-3.5" /></button>
      <button type="button" onClick={onRedo} disabled={!canRedo} className="btn btn-ghost btn-sm" style={{ opacity: canRedo ? 1 : 0.3 }} title={redoTitle}><Redo2 className="w-3.5 h-3.5" /></button>
      <div className="vfd-topbar-divider" />
      <button type="button" onClick={onSave} className="btn btn-primary btn-sm"><Save className="w-3 h-3" /> {saveLabel}</button>
    </div>
  );
}
