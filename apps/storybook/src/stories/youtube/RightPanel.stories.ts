import type { Meta, StoryObj } from '@storybook/svelte';
import RightPanel from 'ui-lib/components/youtube/RightPanel.svelte';

const meta = {
	title: 'YouTube/RightPanel',
	component: RightPanel,
	tags: ['autodocs']
} satisfies Meta<typeof RightPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
