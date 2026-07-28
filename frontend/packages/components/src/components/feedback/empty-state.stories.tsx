import type { Meta, StoryObj } from '@storybook/react';
import { EmptyState } from './empty-state';

const meta: Meta<typeof EmptyState> = {
  title: 'Feedback/EmptyState',
  component: EmptyState,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    title: 'No data available',
    description: 'There is no data to display at this time.',
  },
};

export const WithAction: Story = {
  args: {
    title: 'No projects found',
    description: 'Get started by creating your first project.',
    action: {
      label: 'Create Project',
      onClick: () => {},
    },
  },
};
