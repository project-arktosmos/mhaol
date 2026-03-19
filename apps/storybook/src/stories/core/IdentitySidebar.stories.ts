import type { Meta, StoryObj } from '@storybook/svelte';
import IdentitySidebar from 'ui-lib/components/core/IdentitySidebar.svelte';

const meta = {
	title: 'Core/IdentitySidebar',
	component: IdentitySidebar,
	tags: ['autodocs']
} satisfies Meta<typeof IdentitySidebar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
