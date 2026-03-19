import type { Meta, StoryObj } from '@storybook/svelte';
import LandingNavbar from 'ui-lib/components/landing/LandingNavbar.svelte';

const meta = {
	title: 'Landing/LandingNavbar',
	component: LandingNavbar,
	tags: ['autodocs']
} satisfies Meta<typeof LandingNavbar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
