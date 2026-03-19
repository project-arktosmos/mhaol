import type { Meta, StoryObj } from '@storybook/svelte';
import DiskContent from 'ui-lib/components/settings/DiskContent.svelte';

const meta = {
	title: 'Settings/DiskContent',
	component: DiskContent,
	tags: ['autodocs']
} satisfies Meta<typeof DiskContent>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
