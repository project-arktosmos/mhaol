import type { Meta, StoryObj } from '@storybook/svelte';
import FeatureCard from 'ui-lib/components/landing/FeatureCard.svelte';

const meta = {
	title: 'Landing/FeatureCard',
	component: FeatureCard,
	tags: ['autodocs']
} satisfies Meta<typeof FeatureCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { args: { title: 'Fast Downloads', description: 'Download media at blazing speed', icon: '🚀' } };
