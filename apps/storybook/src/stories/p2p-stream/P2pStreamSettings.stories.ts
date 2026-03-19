import type { Meta, StoryObj } from '@storybook/svelte';
import P2pStreamSettings from 'ui-lib/components/p2p-stream/P2pStreamSettings.svelte';

const meta = {
	title: 'P2pStream/P2pStreamSettings',
	component: P2pStreamSettings,
	tags: ['autodocs']
} satisfies Meta<typeof P2pStreamSettings>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
