# Approval Components · 审批工作区

审批操作工作区公共组件，用于业务模块 TopBar 右侧的审批入口。

## 组件清单

| 组件                | 说明                                           |
| ------------------- | ---------------------------------------------- |
| `ApprovalWorkspace` | **推荐** — 组合触发按钮 + Sheet 面板，开箱即用 |
| `ApprovalTrigger`   | 仅触发按钮（带徽标），需配合外部 Sheet 使用    |
| `ApprovalPanel`     | 审批面板内容（Tab + 列表 + 搜索 + 空状态）     |
| `ApprovalCard`      | 审批单据卡片                                   |

## 快速开始

### 方式一：ApprovalWorkspace（推荐）

直接替换模块 Layout 中 TopBar 的审批按钮：

```tsx
import { ApprovalWorkspace } from '@aliothstudio/components';
import type { ApprovalItem } from '@aliothstudio/components';

// 在你的 Layout 组件中
const [approvalItems, setApprovalItems] = useState<ApprovalItem[]>([
  {
    id: 1,
    title: 'Q2 采购计划审批',
    applicant: '王主管',
    dept: '采购部',
    code: 'PR-2024-001',
    status: 'pending',
    time: '今天 09:30',
  },
  // ...
]);

const pendingCount = approvalItems.filter((i) => i.status === 'pending').length;

// TopBar actions
<TopBar
  actions={
    <div className="flex items-center gap-1">
      <ApprovalWorkspace
        items={approvalItems}
        pendingCount={pendingCount}
        onApprove={(id) => {
          // 调用 API 通过后更新本地状态
          setApprovalItems((prev) =>
            prev.map((i) => (i.id === id ? { ...i, status: 'approved' } : i)),
          );
        }}
        onReject={(id) => {
          setApprovalItems((prev) =>
            prev.map((i) => (i.id === id ? { ...i, status: 'rejected' } : i)),
          );
        }}
        onItemClick={(item) => {
          // 跳转审批详情页
          navigate(`/approvals/${item.id}`);
        }}
      />
      {/* 其他公共按钮... */}
    </div>
  }
/>;
```

### 方式二：自定义组合（高级）

如需完全自定义 Sheet 行为：

```tsx
import { ApprovalTrigger, ApprovalPanel } from "@aliothstudio/components";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@aliothstudio/components";

const [open, setOpen] = useState(false);

<ApprovalTrigger pendingCount={3} onClick={() => setOpen(true)} />

<Sheet open={open} onOpenChange={setOpen}>
  <SheetContent side="right" className="p-0 w-[480px]">
    <SheetHeader className="border-b px-6 py-4">
      <SheetTitle>审批工作区</SheetTitle>
    </SheetHeader>
    <ApprovalPanel items={items} onApprove={handleApprove} onReject={handleReject} />
  </SheetContent>
</Sheet>
```

## 类型定义

```ts
interface ApprovalItem {
  id: string | number;
  title: string;
  applicant: string;
  dept?: string;
  code?: string;
  type?: string;
  status: 'pending' | 'approved' | 'rejected';
  time: string;
  avatar?: string;
}
```

## 状态徽章规范

| 状态   | 颜色            |
| ------ | --------------- |
| 待审批 | amber（琥珀色） |
| 已通过 | green（绿色）   |
| 已驳回 | red（红色）     |

符合 Gateway 设计规范 §9.2。
