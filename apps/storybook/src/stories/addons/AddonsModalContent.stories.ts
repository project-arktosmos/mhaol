import type { Meta, StoryObj } from '@storybook/svelte';
import AddonsModalContent from 'ui-lib/components/addons/AddonsModalContent.svelte';

const meta = {
	title: 'Addons/AddonsModalContent',
	component: AddonsModalContent,
	tags: ['autodocs']
} satisfies Meta<typeof AddonsModalContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
