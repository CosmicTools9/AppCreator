import type { Meta, StoryObj } from '@storybook/react';
import { Checkbox } from './checkbox';
import { Label } from './label';

const meta: Meta<typeof Checkbox> = {
  title: 'UI/Checkbox',
  component: Checkbox,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => (
    <div className="flex items-center space-x-2">
      <Checkbox id="terms" />
      <Label htmlFor="terms">接受条款</Label>
    </div>
  ),
};

export const Checked: Story = {
  render: () => (
    <div className="flex items-center space-x-2">
      <Checkbox id="terms-checked" defaultChecked />
      <Label htmlFor="terms-checked">已选中</Label>
    </div>
  ),
};

export const Disabled: Story = {
  render: () => (
    <div className="flex items-center space-x-2">
      <Checkbox id="terms-disabled" disabled />
      <Label htmlFor="terms-disabled" className="text-muted-foreground">
        禁用状态
      </Label>
    </div>
  ),
};

export const DisabledChecked: Story = {
  render: () => (
    <div className="flex items-center space-x-2">
      <Checkbox id="terms-disabled-checked" disabled defaultChecked />
      <Label htmlFor="terms-disabled-checked" className="text-muted-foreground">
        禁用且已选中
      </Label>
    </div>
  ),
};

export const Group: Story = {
  render: () => (
    <div className="space-y-3">
      <div className="flex items-center space-x-2">
        <Checkbox id="c1" />
        <Label htmlFor="c1">选项一</Label>
      </div>
      <div className="flex items-center space-x-2">
        <Checkbox id="c2" defaultChecked />
        <Label htmlFor="c2">选项二</Label>
      </div>
      <div className="flex items-center space-x-2">
        <Checkbox id="c3" />
        <Label htmlFor="c3">选项三</Label>
      </div>
    </div>
  ),
};
