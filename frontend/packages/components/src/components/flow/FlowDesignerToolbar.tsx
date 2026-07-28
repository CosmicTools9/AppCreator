import * as React from 'react';
import { Play, Check, Users, Diamond, Layers, Mail, GitBranch, ChevronDown } from 'lucide-react';
import type { NodeTypeConfig } from './types';
import { NODE_TYPES } from './utils';
function paletteIcon(type: string): React.ReactNode {
  switch (type) {
    case 'start': return <Play className="w-4 h-4" />;
    case 'end': return <Check className="w-4 h-4" />;
    case 'approval': return <Users className="w-4 h-4" />;
    case 'condition': return <Diamond className="w-4 h-4" />;
    case 'parallel': return <Layers className="w-4 h-4" />;
    case 'cc': return <Mail className="w-4 h-4" />;
    case 'branch': return <GitBranch className="w-4 h-4" />;
    case 'subflow': return <Layers className="w-4 h-4" />;
    default: return <ChevronDown className="w-4 h-4" />;
  }
}
export interface FlowNodePaletteProps { open: boolean; nodeTypes?: NodeTypeConfig[]; onDragStart: (type: string) => void; }
export function FlowNodePalette({ open, nodeTypes = NODE_TYPES, onDragStart }: FlowNodePaletteProps): React.ReactElement {
  return (
    <div className="shrink-0 overflow-hidden transition-all duration-200 border-r" style={{ width: open ? 220 : 0 }}>
      <div className="vfd-palette" style={{ width: 220 }}>
        <div className="vfd-palette-list">
          {nodeTypes.map((nt) => (
            <div key={nt.type} draggable onDragStart={() => onDragStart(nt.type)} className="vfd-palette-item">
              <div className={`vfd-palette-icon ${nt.type}`}>{paletteIcon(nt.type)}</div>
              <div>
                <div style={{ fontWeight: 500, fontSize: 13 }}>{nt.label}</div>
                <div style={{ fontSize: 10.5, color: 'hsl(var(--muted-foreground))' }}>{nt.desc}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
export { NODE_TYPES };
