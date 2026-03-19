import type { Meta, StoryObj } from '@storybook/svelte';
import Footer from 'ui-lib/components/landing/Footer.svelte';

const meta = {
	title: 'Landing/Footer',
	component: Footer,
	tags: ['autodocs']
} satisfies Meta<typeof Footer>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
