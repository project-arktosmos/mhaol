import type { Meta, StoryObj } from '@storybook/svelte';
import IdentityModalContent from 'ui-lib/components/identity/IdentityModalContent.svelte';

const meta = {
	title: 'Identity/IdentityModalContent',
	component: IdentityModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof IdentityModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
