import type { Meta, StoryObj } from '@storybook/react';
import { Stepper } from './stepper';

const meta: Meta<typeof Stepper> = {
  title: 'UI/Stepper',
  component: Stepper,
  parameters: {
    layout: 'centered',
  },
  tags: ['autodocs'],
};

export default meta;
type Story = StoryObj<typeof meta>;

const steps = [
  { id: '1', label: 'Account', description: 'Create your account' },
  { id: '2', label: 'Profile', description: 'Set up your profile' },
  { id: '3', label: 'Review', description: 'Review your details' },
  { id: '4', label: 'Complete', description: 'Finish setup' },
];

export const FirstStep: Story = {
  args: {
    steps,
    currentStep: 0,
  },
};

export const SecondStep: Story = {
  args: {
    steps,
    currentStep: 1,
  },
};

export const LastStep: Story = {
  args: {
    steps,
    currentStep: 3,
  },
};
