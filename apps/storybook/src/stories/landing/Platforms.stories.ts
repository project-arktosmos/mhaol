import type { Meta, StoryObj } from '@storybook/svelte';
import Platforms from 'ui-lib/components/landing/Platforms.svelte';

const meta = {
	title: 'Landing/Platforms',
	component: Platforms,
	tags: ['autodocs']
} satisfies Meta<typeof Platforms>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
