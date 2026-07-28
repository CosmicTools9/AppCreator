import type { Meta, StoryObj } from '@storybook/react';
import { Textarea } from './textarea';
import { Label } from './label';

const meta: Meta<typeof Textarea> = {
  title: 'UI/Textarea',
  component: Textarea,
  parameters: {
    layout: 'padded',
  },
  tags: ['autodocs'],
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => (
    <div className="grid w-full gap-1.5">
      <Label htmlFor="message">Message</Label>
      <Textarea id="message" placeholder="Enter your message..." />
    </div>
  ),
};

export const WithValue: Story = {
  render: () => (
    <div className="grid w-full gap-1.5">
      <Label htmlFor="bio">Bio</Label>
      <Textarea
        id="bio"
        defaultValue="I am a frontend developer passionate about open source."
      />
    </div>
  ),
};

export const Disabled: Story = {
  render: () => (
    <div className="grid w-full gap-1.5">
      <Label htmlFor="disabled">Disabled</Label>
      <Textarea id="disabled" disabled placeholder="Cannot input..." />
    </div>
  ),
};

export const Rows: Story = {
  render: () => (
    <div className="grid w-full gap-1.5">
      <Label htmlFor="long">Long text</Label>
      <Textarea id="long" rows={8} placeholder="Supports multi-line input..." />
    </div>
  ),
};
