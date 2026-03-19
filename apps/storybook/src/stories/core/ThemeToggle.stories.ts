import type { Meta, StoryObj } from '@storybook/svelte';
import ThemeToggle from 'ui-lib/components/core/ThemeToggle.svelte';

const meta = {
	title: 'Core/ThemeToggle',
	component: ThemeToggle,
	tags: ['autodocs']
} satisfies Meta<typeof ThemeToggle>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
