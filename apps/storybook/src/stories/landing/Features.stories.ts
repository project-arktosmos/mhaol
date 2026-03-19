import type { Meta, StoryObj } from '@storybook/svelte';
import Features from 'ui-lib/components/landing/Features.svelte';

const meta = {
	title: 'Landing/Features',
	component: Features,
	tags: ['autodocs']
} satisfies Meta<typeof Features>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
