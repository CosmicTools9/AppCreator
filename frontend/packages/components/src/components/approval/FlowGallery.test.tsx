import { describe, it, expect } from 'vitest';
import { render } from '@/test/test-utils';
import { GalleryMiniPreview } from './FlowGallery';
import type { FlowItem } from './FlowGallery';

const flow: FlowItem = { id: 1, name: '并行会签审批', status: 'draft' };

const parallelNodes = [
  { id: 'n0', label: '提交申请', cls: 'start' as const },
  { id: 'n1', label: '并行会签', cls: 'condition' as const },
  { id: 'n2', label: '财务会签', cls: 'approval' as const },
  { id: 'n3', label: '法务会签', cls: 'approval' as const },
  { id: 'n4', label: '会签汇总', cls: 'approval' as const },
  { id: 'n5', label: '审批完成', cls: 'end' as const },
];

const parallelEdges = [
  { from: 'n0', to: 'n1' },
  { from: 'n1', to: 'n2' },
  { from: 'n1', to: 'n3' },
  { from: 'n2', to: 'n4' },
  { from: 'n3', to: 'n4' },
  { from: 'n4', to: 'n5' },
];

describe('GalleryMiniPreview', () => {
  it('renders branch stack when getEdges provides fork topology', () => {
    const { container } = render(
      <GalleryMiniPreview flow={flow} getNodes={() => parallelNodes} getEdges={() => parallelEdges} />,
    );
    const branch = container.querySelector('.gallery-mini-branch');
    expect(branch).not.toBeNull();
    // 两个并行分支节点在同一分支容器内
    const branchText = branch!.textContent ?? '';
    expect(branchText).toContain('财务会签');
    expect(branchText).toContain('法务会签');
    // 汇聚节点在分支容器之外继续渲染
    expect(container.textContent).toContain('会签汇总');
    expect(container.textContent).toContain('审批完成');
  });

  it('falls back to sequential chip chain without getEdges', () => {
    const { container } = render(
      <GalleryMiniPreview flow={flow} getNodes={() => parallelNodes} />,
    );
    expect(container.querySelector('.gallery-mini-branch')).toBeNull();
    expect(container.textContent).toContain('提交申请');
    expect(container.textContent).toContain('审批完成');
  });

  it('uses array index as implicit id when nodes lack id', () => {
    const noIdNodes = parallelNodes.map(({ label, cls }) => ({ label, cls }));
    const indexEdges = [
      { from: '0', to: '1' },
      { from: '1', to: '2' },
      { from: '1', to: '3' },
      { from: '2', to: '4' },
      { from: '3', to: '4' },
      { from: '4', to: '5' },
    ];
    const { container } = render(
      <GalleryMiniPreview flow={flow} getNodes={() => noIdNodes} getEdges={() => indexEdges} />,
    );
    expect(container.querySelector('.gallery-mini-branch')).not.toBeNull();
  });
});
