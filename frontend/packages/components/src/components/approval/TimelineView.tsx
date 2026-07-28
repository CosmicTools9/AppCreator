/**
 * TimelineView — 审批时间线（对齐 v30 TimelineView）
 * 使用 theme.css .timeline/.timeline-row/.timeline-dot/.user-chip-sm 类
 */

import { Check, Clock, UserCheck } from "lucide-react";

export interface TimelineNode {
 nodeName?: string;
 approver?: string;
 time?: string;
 status?: string;
 opinion?: string;
 sla?: number;
}
export function TimelineView({ events }: { events: TimelineNode[] }) {
 return (
  <div className="timeline">
   {events.map((e, i) => {
    const dotClass = e.status === "completed" ? "completed"
     : e.status === "active" ? "active"
      : e.status === "rejected" ? "rejected" : "pending";
    return (
     <div key={i} className="timeline-row">
      <div className={"timeline-dot " + dotClass}>
       {e.status === "completed"
        ? <Check className="w-3 h-3" />
        : e.status === "active"
         ? <Clock className="w-3 h-3" />
         : <span className="chain-dot-num">{i + 1}</span>}
      </div>
      <div className="timeline-content">
       <div className="timeline-row-head">
        <span className="timeline-row-name">{e.nodeName}</span>
        {e.approver ? (
         <span className="user-chip-sm">
          <UserCheck className="w-3 h-3" />
          {e.approver}
         </span>
        ) : null}
        <span className="timeline-row-time">
         {e.time ? e.time.slice(11, 16) : ""}
        </span>
       </div>
       {e.opinion ? <div className="timeline-row-opinion">{e.opinion}</div> : null}
       {e.sla !== undefined ? (
        <div className="sla-progress" style={{ marginTop: 6, maxWidth: 200 }}>
         <div className={`sla-fill ${e.sla > 70 ? "safe" : e.sla > 40 ? "warn" : "danger"}`} style={{ width: e.sla + "%" }} />
        </div>
       ) : null}
      </div>
     </div>
    );
   })}
  </div>
 );
}
