import type { Meta, StoryObj } from '@storybook/svelte';
import Hero from 'ui-lib/components/landing/Hero.svelte';

const meta = {
	title: 'Landing/Hero',
	component: Hero,
	tags: ['autodocs']
} satisfies Meta<typeof Hero>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
