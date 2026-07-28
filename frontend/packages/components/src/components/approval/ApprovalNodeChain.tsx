
/**
 * ApprovalNodeChain — 审批节点链展示（对齐 v30 ApprovalNodeChain）
 * 使用 theme.css .chain/.chain-node/.chain-dot/.chain-label 类
 */

import { Check, Clock } from "lucide-react";

type TimelineNodeStatus = "completed" | "active" | "pending" | "rejected";
export interface ChainNode {
 id: string;
 label: string;
 name?: string;
 status: TimelineNodeStatus;
}

export function ApprovalNodeChain({ nodes }: { nodes: ChainNode[] }) {
 return (
  <div className="chain">
   {nodes.map((node, i) => {
    const isCompleted = node.status === "completed";
    const isActive = node.status === "active";
    return (
     <span key={node.id} className="flex items-center" style={{ gap: 2 }}>
      <div className={`chain-node ${node.status}`}>
       <div className={`chain-dot ${node.status}`}>
        {isCompleted ? <Check className="w-3 h-3" /> : isActive ? <Clock className="w-3 h-3" /> : <span className="chain-dot-num">{i + 1}</span>}
       </div>
       <div style={{ minWidth: 0 }}>
        <div className="chain-label">
         <span className="chain-label-text">{node.label}</span>
        </div>
        {node.name ? <div style={{ fontSize: 11, color: "hsl(var(--muted-foreground))" }}>{node.name}</div> : null}
       </div>
      </div>
      {i < nodes.length - 1 ? <svg className="chain-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M5 12h14M12 5l7 7-7 7" /></svg> : null}
     </span>
    );
   })}
  </div>
 );
}
