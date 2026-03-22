import type { Meta, StoryObj } from '@storybook/svelte';
import BrowseDetailPanel from 'ui-lib/components/browse/BrowseDetailPanel.svelte';

const meta = {
	title: 'Browse/BrowseDetailPanel',
	component: BrowseDetailPanel,
	tags: ['autodocs']
} satisfies Meta<typeof BrowseDetailPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
