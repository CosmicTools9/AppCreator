import type { Meta, StoryObj } from '@storybook/react';
import { Alert, AlertTitle, AlertDescription } from './alert';
import { Terminal, AlertCircle, Info, CheckCircle2 } from 'lucide-react';

const meta: Meta<typeof Alert> = {
  title: 'UI/Alert',
  component: Alert,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => (
    <Alert>
      <Terminal className="h-4 w-4" />
      <AlertTitle>提示</AlertTitle>
      <AlertDescription>这是一条默认的提示信息。</AlertDescription>
    </Alert>
  ),
};

export const Destructive: Story = {
  render: () => (
    <Alert variant="destructive">
      <AlertCircle className="h-4 w-4" />
      <AlertTitle>错误</AlertTitle>
      <AlertDescription>操作失败，请检查网络连接后重试。</AlertDescription>
    </Alert>
  ),
};

export const InfoVariant: Story = {
  render: () => (
    <Alert>
      <Info className="h-4 w-4" />
      <AlertTitle>信息</AlertTitle>
<AlertDescription>新功能已上线，请查看更新日志。</AlertDescription>
    </Alert>
  ),
};

export const Success: Story = {
  render: () => (
    <Alert className="border-green-500/50 text-success/80 dark:text-green-300">
      <CheckCircle2 className="h-4 w-4 text-success dark:text-green-400" />
      <AlertTitle>成功</AlertTitle>
      <AlertDescription>数据已成功保存。</AlertDescription>
    </Alert>
  ),
};

export const WithoutDescription: Story = {
  render: () => (
    <Alert>
      <Info className="h-4 w-4" />
      <AlertTitle>仅标题的提示</AlertTitle>
    </Alert>
  ),
};
