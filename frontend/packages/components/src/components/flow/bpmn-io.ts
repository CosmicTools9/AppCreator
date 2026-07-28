/**
 * BPMN 2.0 import/export for FlowDesigner
 *
 * Maps between FlowDesigner's internal node model and BPMN 2.0 XML.
 * Supported: startEvent, endEvent, userTask (approval), exclusiveGateway (condition),
 * parallelGateway (parallel), callActivity (subflow).
 */

import type { FlowNode } from './types';

// ── Helpers ─────────────────────────────────────

function el(tag: string, attrs: Record<string, string> = {}, children: string[] = []): string {
  const attrStr = Object.entries(attrs)
    .map(([k, v]) => ` ${k}="${v.replace(/"/g, '&quot;')}"`)
    .join('');
  return children.length
    ? `<${tag}${attrStr}>${children.join('')}</${tag}>`
    : `<${tag}${attrStr} />`;
}

function escXml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

// ── Export: FlowNode[] → BPMN XML ───────────────

export function exportBpmn(nodes: FlowNode[], processName = 'approval-process'): string {
  const bpmnNodes: string[] = [];
  const bpmnFlows: string[] = [];

  nodes.forEach((node, i) => {
    const id = `Node_${i}`;
    switch (node.type) {
      case 'start':
        bpmnNodes.push(el('bpmn:startEvent', { id }));
        break;
      case 'end':
        bpmnNodes.push(el('bpmn:endEvent', { id }));
        break;
      case 'approval':
        bpmnNodes.push(el('bpmn:userTask', { id, name: escXml(node.label) }));
        break;
      case 'condition':
        bpmnNodes.push(
          el('bpmn:exclusiveGateway', {
            id,
            ...(node.label ? { name: escXml(node.label) } : {}),
            gatewayDirection: 'Diverging',
          }),
        );
        break;
      case 'cc':
        bpmnNodes.push(el('bpmn:task', { id, name: `CC: ${escXml(node.label)}` }));
        break;
      case 'parallel':
        bpmnNodes.push(
          el('bpmn:parallelGateway', {
            id,
            name: escXml(node.label),
            gatewayDirection: 'Diverging',
          }),
        );
        break;
      case 'subflow':
        bpmnNodes.push(
          el('bpmn:callActivity', {
            id,
            name: escXml(node.label),
            calledElement: node.target || 'sub-process',
          }),
        );
        break;
      default:
        bpmnNodes.push(el('bpmn:task', { id, name: escXml(node.label) }));
    }
  });

  // Generate sequence flows from node.next edges
  nodes.forEach((node, i) => {
    const nexts = node.next ?? [];
    if (nexts.length === 0 && i < nodes.length - 1) {
      // Linear fallback
      bpmnFlows.push(
        el('bpmn:sequenceFlow', {
          id: `Flow_${i}_${i + 1}`,
          sourceRef: `Node_${i}`,
          targetRef: `Node_${i + 1}`,
        }),
      );
    } else {
      nexts.forEach((edge, j) => {
        bpmnFlows.push(
          el('bpmn:sequenceFlow', {
            id: `Flow_${i}_${j}`,
            sourceRef: `Node_${i}`,
            targetRef: `Node_${edge.to}`,
            ...(edge.label ? { name: escXml(edge.label) } : {}),
          }),
        );
      });
    }
  });

  return `<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
  xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI"
  xmlns:dc="http://www.omg.org/spec/DD/20100524/DC"
  targetNamespace="http://alioth.app/approval">
  <bpmn:process id="${escXml(processName)}" isExecutable="false">
    ${bpmnNodes.join('\n    ')}
    ${bpmnFlows.join('\n    ')}
  </bpmn:process>
</bpmn:definitions>`;
}

// ── Import: BPMN XML → FlowNode[] ───────────────

const BPMN_TYPE_MAP: Record<string, string> = {
  startEvent: 'start',
  endEvent: 'end',
  userTask: 'approval',
  exclusiveGateway: 'condition',
  parallelGateway: 'parallel',
  callActivity: 'subflow',
  task: 'cc',
};

export function importBpmn(xml: string): { nodes: FlowNode[]; error?: string } {
  let doc: Document;
  try {
    doc = new DOMParser().parseFromString(xml, 'text/xml');
    const parseErr = doc.querySelector('parsererror');
    if (parseErr) return { nodes: [], error: parseErr.textContent ?? 'Invalid XML' };
  } catch (e) {
    return { nodes: [], error: String(e) };
  }

  const processEl = doc.querySelector('bpmn\\:process, process');
  if (!processEl) return { nodes: [], error: 'No <bpmn:process> found' };

  const taskElements = processEl.querySelectorAll(
    'bpmn\\:startEvent, bpmn\\:endEvent, bpmn\\:userTask, bpmn\\:exclusiveGateway, bpmn\\:parallelGateway, bpmn\\:callActivity, bpmn\\:task, startEvent, endEvent, userTask, exclusiveGateway, parallelGateway, callActivity, task',
  );
  const flowElements = processEl.querySelectorAll('bpmn\\:sequenceFlow, sequenceFlow');

  // Collect flow edges for topology
  const sourceTargetMap: Map<string, string> = new Map();
  flowElements.forEach((flow) => {
    const src = flow.getAttribute('sourceRef') ?? '';
    const tgt = flow.getAttribute('targetRef') ?? '';
    if (src && tgt) sourceTargetMap.set(src, tgt);
  });

  const nodes: FlowNode[] = [];
  const idToIdx: Map<string, number> = new Map();

  taskElements.forEach((el) => {
    const localName = el.localName || el.tagName.split(':').pop() || '';
    const type = BPMN_TYPE_MAP[localName] ?? 'task';
    const bpmnId = el.getAttribute('id') ?? `n_${nodes.length}`;
    const name = el.getAttribute('name') ?? localName;

    const node: FlowNode = {
      id: `n-${nodes.length}`,
      type,
      label: name,
      x: 60 + (nodes.length % 4) * 220,
      y: 60 + Math.floor(nodes.length / 4) * 160,
    };
    idToIdx.set(bpmnId, nodes.length);
    nodes.push(node);
  });

  // Build edges
  flowElements.forEach((flow) => {
    const src = flow.getAttribute('sourceRef') ?? '';
    const tgt = flow.getAttribute('targetRef') ?? '';
    const from = idToIdx.get(src);
    const to = idToIdx.get(tgt);
    if (from !== undefined && to !== undefined) {
      const node = nodes[from];
      if (node) {
        node.next = [...(node.next ?? []), { to }];
      }
    }
  });

  return { nodes };
}
